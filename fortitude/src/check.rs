use crate::ast::FortitudeNode;
use crate::cli::{CheckArgs, GlobalConfigArgs, FORTRAN_EXTS};
use crate::diagnostics::{Diagnostics, FixMap};
use crate::fix::{fix_file, FixResult};
use crate::fs;
use crate::message::DiagnosticMessage;
use crate::printer::{Flags as PrinterFlags, Printer};
use crate::registry::AsRule;
use crate::rule_selector::{PreviewOptions, RuleSelector, Specificity};
use crate::rules::Rule;
use crate::rules::{error::ioerror::IoError, AstRuleEnum, PathRuleEnum, TextRuleEnum};
use crate::settings::{
    FixMode, OutputFormat, PreviewMode, ProgressBar, Settings, UnsafeFixes, DEFAULT_SELECTORS,
};

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use ruff_diagnostics::Diagnostic;
use ruff_source_file::{Locator, SourceFile, SourceFileBuilder};
use ruff_text_size::TextRange;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use strum::IntoEnumIterator;
use toml::Table;
use tree_sitter::Parser;
use walkdir::WalkDir;

// These are just helper structs to let us quickly work out if there's
// a fortitude section in an fpm.toml file
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Fpm {
    extra: Option<Extra>,
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Extra {
    fortitude: Option<CheckSection>,
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct CheckSection {
    check: Option<CheckArgs>,
}

// Adapted from ruff
fn parse_fpm_toml<P: AsRef<Path>>(path: P) -> Result<Fpm> {
    let contents = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.as_ref().display()))
}

pub fn fortitude_enabled<P: AsRef<Path>>(path: P) -> Result<bool> {
    let fpm = parse_fpm_toml(path)?;
    Ok(fpm.extra.and_then(|extra| extra.fortitude).is_some())
}

/// Return the path to the `fpm.toml` or `fortitude.toml` file in a given
/// directory. Adapated from ruff
pub fn settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    // Check for `.fortitude.toml`.
    let fortitude_toml = path.as_ref().join(".fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fortitude.toml`.
    let fortitude_toml = path.as_ref().join("fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fpm.toml`.
    let fpm_toml = path.as_ref().join("fpm.toml");
    if fpm_toml.is_file() && fortitude_enabled(&fpm_toml)? {
        return Ok(Some(fpm_toml));
    }

    Ok(None)
}

/// Find the path to the `fpm.toml` or `fortitude.toml` file, if such a file
/// exists. Adapated from ruff
pub fn find_settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    for directory in path.as_ref().ancestors() {
        if let Some(settings) = settings_toml(directory)? {
            return Ok(Some(settings));
        }
    }
    Ok(None)
}

/// Read either the "extra.fortitude" table from "fpm.toml", or the
/// whole "fortitude.toml" file
fn from_toml_subsection<P: AsRef<Path>>(path: P) -> Result<CheckSection> {
    let config_str = if path.as_ref().ends_with("fpm.toml") {
        let config = std::fs::read_to_string(path)?.parse::<Table>()?;

        // Unwrap should be ok here because we've already checked this
        // file has these tables
        let extra = &config["extra"].as_table().unwrap();
        let fortitude = &extra["fortitude"].as_table().unwrap();
        fortitude.to_string()
    } else {
        std::fs::read_to_string(path)?
    };

    let config: CheckSection = toml::from_str(&config_str)?;

    Ok(config)
}

/// Resolve `--foo` and `--no-foo` arguments
fn resolve_bool_arg(yes: Option<bool>, no: Option<bool>) -> Option<bool> {
    let yes = yes.unwrap_or_default();
    let no = no.unwrap_or_default();
    match (yes, no) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (..) => unreachable!("Clap should make this impossible"),
    }
}

// This is our "known good" intermediate settings struct after we've
// read the config file, but before we've overridden it from the CLI
#[derive(Default, Debug)]
pub struct CheckSettings {
    pub files: Vec<PathBuf>,
    pub ignore: Vec<RuleSelector>,
    pub select: Option<Vec<RuleSelector>>,
    pub extend_select: Vec<RuleSelector>,
    pub line_length: usize,
    pub file_extensions: Vec<String>,
    pub fix: bool,
    pub fix_only: bool,
    pub show_fixes: bool,
    pub unsafe_fixes: UnsafeFixes,
    pub output_format: OutputFormat,
    pub progress_bar: ProgressBar,
    pub preview: PreviewMode,
}

/// Read either fpm.toml or fortitude.toml into our "known good" file
/// settings struct
fn parse_config_file(config_file: &Option<PathBuf>) -> Result<CheckSettings> {
    let filename = match config_file {
        Some(filename) => filename.clone(),
        None => match find_settings_toml(".")? {
            Some(filename) => filename,
            None => {
                return Ok(CheckSettings::default());
            }
        },
    };

    let settings = match from_toml_subsection(filename)?.check {
        Some(value) => CheckSettings {
            files: value.files.unwrap_or(vec![PathBuf::from(".")]),
            ignore: value.ignore.unwrap_or_default(),
            select: value.select,
            extend_select: value.extend_select.unwrap_or_default(),
            line_length: value.line_length.unwrap_or(Settings::default().line_length),
            file_extensions: value
                .file_extensions
                .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect_vec()),
            fix: resolve_bool_arg(value.fix, value.no_fix).unwrap_or_default(),
            fix_only: resolve_bool_arg(value.fix_only, value.no_fix_only).unwrap_or_default(),
            show_fixes: resolve_bool_arg(value.show_fixes, value.no_show_fixes).unwrap_or_default(),
            unsafe_fixes: resolve_bool_arg(value.unsafe_fixes, value.no_unsafe_fixes)
                .map(UnsafeFixes::from)
                .unwrap_or_default(),
            output_format: value.output_format.unwrap_or_default(),
            progress_bar: value.progress_bar.unwrap_or_default(),
            preview: resolve_bool_arg(value.preview, value.no_preview)
                .map(PreviewMode::from)
                .unwrap_or_default(),
        },
        None => CheckSettings::default(),
    };
    Ok(settings)
}

/// Get the list of active rules for this session.
fn ruleset(args: RuleSelection, preview: &PreviewMode) -> anyhow::Result<Vec<Rule>> {
    let preview = PreviewOptions {
        mode: *preview,
        require_explicit: false,
    };

    // The select_set keeps track of which rules have been selected.
    let mut select_set: BTreeSet<Rule> = if args.select.is_none() {
        DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect()
    } else {
        BTreeSet::default()
    };

    for spec in Specificity::iter() {
        // Iterate over rule selectors in order of specificity.
        for selector in args
            .select
            .iter()
            .flatten()
            .chain(args.extend_select.iter())
            .filter(|s| s.specificity() == spec)
        {
            for rule in selector.rules(&preview) {
                select_set.insert(rule);
            }
        }

        for selector in args.ignore.iter().filter(|s| s.specificity() == spec) {
            for rule in selector.rules(&preview) {
                select_set.remove(&rule);
            }
        }
    }

    let rules = select_set.into_iter().collect_vec();

    Ok(rules)
}

/// Helper function used with `filter` to select only paths that end in a Fortran extension.
/// Includes non-standard extensions, as these should be reported.
fn filter_fortran_extensions<S: AsRef<str>>(path: &Path, extensions: &[S]) -> bool {
    if let Some(ext) = path.extension() {
        // Can't use '&[&str].contains()', as extensions are of type OsStr
        extensions.iter().any(|x| x.as_ref() == ext)
    } else {
        false
    }
}

/// Expand the input list of files to include all Fortran files.
fn get_files<S: AsRef<str>>(files_in: &Vec<PathBuf>, extensions: &[S]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for path in files_in {
        if path.is_dir() {
            paths.extend(
                WalkDir::new(path)
                    .min_depth(1)
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .map(|x| x.path().to_path_buf())
                    .filter(|x| filter_fortran_extensions(x.as_path(), extensions)),
            );
        } else {
            paths.push(path.to_path_buf());
        }
    }
    paths
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_file(
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    fix_mode: FixMode,
    unsafe_fixes: UnsafeFixes,
) -> anyhow::Result<Diagnostics> {
    let (messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
            unsafe_fixes,
        ) {
            if !fixed.is_empty() {
                match fix_mode {
                    FixMode::Apply => {
                        let mut out_file = File::create(path)?;
                        out_file.write_all(transformed.source_text().as_bytes())?;
                    }
                    // TODO: diff
                    FixMode::Diff => {}
                    FixMode::Generate => {}
                }
            }

            (result, fixed)
        } else {
            // Failed to fix, so just lint the original source
            let result = check_only_file(
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                file,
                settings,
            )?;
            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
        )?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    Ok(Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    })
}

/// Parse a file, check it for issues, and return the report.
pub(crate) fn check_only_file(
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
) -> anyhow::Result<Vec<DiagnosticMessage>> {
    let mut violations = Vec::new();

    for rule in path_rules {
        if let Some(violation) = rule.check(settings, path) {
            violations.push(violation);
        }
    }

    // Perform plain text analysis
    for rule in text_rules {
        violations.extend(rule.check(settings, file));
    }

    // Perform AST analysis
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;
    let tree = parser
        .parse(file.source_text(), None)
        .context("Failed to parse")?;
    for node in tree.root_node().named_descendants() {
        if let Some(rules) = ast_entrypoints.get(node.kind()) {
            for rule in rules {
                if let Some(violation) = rule.check(settings, &node, file) {
                    for v in violation {
                        violations.push(v);
                    }
                }
            }
        }
    }

    Ok(violations
        .into_iter()
        .map(|v| DiagnosticMessage::from_ruff(file, v))
        .collect_vec())
}

const MAX_ITERATIONS: usize = 100;

pub type FixTable = FxHashMap<Rule, usize>;

pub struct FixerResult<'a> {
    /// The result returned by the linter, after applying any fixes.
    pub result: Vec<DiagnosticMessage>,
    /// The resulting source code, after applying any fixes.
    pub transformed: Cow<'a, SourceFile>,
    /// The number of fixes applied for each [`Rule`].
    pub fixed: FixTable,
}

pub(crate) fn check_and_fix_file<'a>(
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &'a SourceFile,
    settings: &Settings,
    unsafe_fixes: UnsafeFixes,
) -> anyhow::Result<FixerResult<'a>> {
    let mut transformed = Cow::Borrowed(file);

    // Track the number of fixed errors across iterations.
    let mut fixed = FxHashMap::default();

    // As an escape hatch, bail after 100 iterations.
    let mut iterations = 0;

    // Track whether the _initial_ source code is valid syntax.
    // TODO(peter): implement this
    #[allow(unused_variables, unused_mut)]
    let mut is_valid_syntax = false;

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;

    // Continuously fix until the source code stabilizes.
    loop {
        let mut violations = Vec::new();

        // Map row and column locations to byte slices (lazily).
        let locator = Locator::new(transformed.source_text());

        // No fixes on file path
        for rule in path_rules {
            if let Some(violation) = rule.check(settings, path) {
                violations.push(violation);
            }
        }

        // Perform plain text analysis
        for rule in text_rules {
            violations.extend(rule.check(settings, &transformed));
        }

        // TODO: check for syntax errors on first pass, so we can know
        // if we've introduced them
        let tree = parser
            .parse(transformed.source_text(), None)
            .context("Failed to parse")?;

        // Perform AST analysis
        for node in tree.root_node().named_descendants() {
            if let Some(rules) = ast_entrypoints.get(node.kind()) {
                for rule in rules {
                    if let Some(violation) = rule.check(settings, &node, &transformed) {
                        for v in violation {
                            violations.push(v);
                        }
                    }
                }
            }
        }

        // Apply fix
        if let Some(FixResult {
            code: fixed_contents,
            fixes: applied,
            ..
        }) = fix_file(
            &violations,
            &locator,
            unsafe_fixes,
            path.to_string_lossy().as_ref(),
        ) {
            if iterations < MAX_ITERATIONS {
                // Count the number of fixed errors
                for (rule, count) in applied {
                    *fixed.entry(rule).or_default() += count;
                }

                transformed = Cow::Owned(fixed_contents);

                iterations += 1;

                // Re-run the linter pass
                continue;
            }

            report_failed_to_converge_error(path, transformed.source_text(), &violations);
        };

        return Ok(FixerResult {
            result: violations
                .into_iter()
                .map(|v| DiagnosticMessage::from_ruff(&transformed, v))
                .collect_vec(),
            transformed,
            fixed,
        });
    }
}

fn collect_rule_codes(rules: impl IntoIterator<Item = Rule>) -> String {
    rules
        .into_iter()
        .map(|rule| rule.noqa_code().to_string())
        .sorted_unstable()
        .dedup()
        .join(", ")
}

#[allow(clippy::print_stderr)]
fn report_failed_to_converge_error(path: &Path, transformed: &str, diagnostics: &[Diagnostic]) {
    let codes = collect_rule_codes(diagnostics.iter().map(|diagnostic| diagnostic.kind.rule()));
    if cfg!(debug_assertions) {
        eprintln!(
            "{}{} Failed to converge after {} iterations in `{}` with rule codes {}:---\n{}\n---",
            "debug error".red().bold(),
            ":".bold(),
            MAX_ITERATIONS,
            fs::relativize_path(path),
            codes,
            transformed,
        );
    } else {
        eprintln!(
            r#"
{}{} Failed to converge after {} iterations.

This indicates a bug in fortitude. If you could open an issue at:

    https://github.com/PlasmaFAIR/fortitude/issues/new?title=%5BInfinite%20loop%5D

...quoting the contents of `{}`, the rule codes {}, along with the `fpm.toml` settings and executed command, we'd be very appreciative!
"#,
            "error".red().bold(),
            ":".bold(),
            MAX_ITERATIONS,
            fs::relativize_path(path),
            codes
        );
    }
}

/// Wrapper around `std::fs::read_to_string` with some extra error
/// checking.
///
/// Check that the file length is representable as `u32` so
/// that we don't need to check when converting tree-sitter offsets
/// (usize) into ruff offsets (u32)
pub(crate) fn read_to_string(path: &Path) -> std::io::Result<String> {
    let metadata = path.metadata()?;
    let file_length = metadata.len();

    if TryInto::<u32>::try_into(file_length).is_err() {
        #[allow(non_snake_case)]
        let length_in_GiB = file_length as f64 / 1024.0 / 1024.0 / 1024.0;
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("larger than maximum 4 GiB ({length_in_GiB} GiB)"),
        ));
    }
    std::fs::read_to_string(path)
}

pub(crate) fn rules_to_path_rules(rules: &[Rule]) -> Vec<PathRuleEnum> {
    rules
        .iter()
        .filter_map(|rule| match TryFrom::try_from(*rule) {
            Ok(path) => Some(path),
            _ => None,
        })
        .collect_vec()
}

pub(crate) fn rules_to_text_rules(rules: &[Rule]) -> Vec<TextRuleEnum> {
    rules
        .iter()
        .filter_map(|rule| match TryFrom::try_from(*rule) {
            Ok(text) => Some(text),
            _ => None,
        })
        .collect_vec()
}

/// Create a mapping of AST entrypoints to lists of the rules and codes that operate on them.
pub(crate) fn ast_entrypoint_map<'a>(rules: &[Rule]) -> BTreeMap<&'a str, Vec<AstRuleEnum>> {
    let ast_rules: Vec<AstRuleEnum> = rules
        .iter()
        .filter_map(|rule| match TryFrom::try_from(*rule) {
            Ok(ast) => Some(ast),
            _ => None,
        })
        .collect();

    let mut map: BTreeMap<&'a str, Vec<_>> = BTreeMap::new();
    for rule in ast_rules {
        for entrypoint in rule.entrypoints() {
            match map.get_mut(entrypoint) {
                Some(rule_vec) => {
                    rule_vec.push(rule);
                }
                None => {
                    map.insert(entrypoint, vec![rule]);
                }
            }
        }
    }
    map
}

// Taken from Ruff
#[derive(Clone, Debug, Default)]
pub struct RuleSelection {
    pub select: Option<Vec<RuleSelector>>,
    pub ignore: Vec<RuleSelector>,
    pub extend_select: Vec<RuleSelector>,
}

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckArgs, global_options: &GlobalConfigArgs) -> Result<ExitCode> {
    // First we need to find and read any config file
    let file_settings = parse_config_file(&global_options.config_file)?;
    // Now, we can override settings from the config file with options
    // from the CLI
    let files = &args.files.unwrap_or(file_settings.files);
    let file_extensions = &args
        .file_extensions
        .unwrap_or(file_settings.file_extensions);

    let settings = Settings {
        line_length: args.line_length.unwrap_or(file_settings.line_length),
    };

    let rule_selection = RuleSelection {
        select: args.select.or(file_settings.select),
        // TODO: CLI ignore should _extend_ file ignore
        ignore: args.ignore.unwrap_or(file_settings.ignore),
        extend_select: args.extend_select.unwrap_or(file_settings.extend_select),
    };

    let output_format = args.output_format.unwrap_or(file_settings.output_format);
    let preview_mode = resolve_bool_arg(args.preview, args.no_preview)
        .map(PreviewMode::from)
        .unwrap_or(file_settings.preview);

    let mut progress_bar = args.progress_bar.unwrap_or(file_settings.progress_bar);
    // Override progress bar settings if not using colour terminal
    if progress_bar == ProgressBar::Fancy && !colored::control::SHOULD_COLORIZE.should_colorize() {
        progress_bar = ProgressBar::Ascii;
    }

    let fix = resolve_bool_arg(args.fix, args.no_fix).unwrap_or(file_settings.fix);
    let fix_only =
        resolve_bool_arg(args.fix_only, args.no_fix_only).unwrap_or(file_settings.fix_only);
    let unsafe_fixes = resolve_bool_arg(args.unsafe_fixes, args.no_unsafe_fixes)
        .map(UnsafeFixes::from)
        .unwrap_or(file_settings.unsafe_fixes);

    let show_fixes =
        resolve_bool_arg(args.show_fixes, args.no_show_fixes).unwrap_or(file_settings.show_fixes);

    // Fix rules are as follows:
    // - By default, generate all fixes, but don't apply them to the filesystem.
    // - If `--fix` or `--fix-only` is set, apply applicable fixes to the filesystem (or
    //   print them to stdout, if we're reading from stdin).
    // - If `--diff` or `--fix-only` are set, don't print any violations (only applicable fixes)
    // - By default, applicable fixes only include [`Applicablility::Automatic`], but if
    //   `--unsafe-fixes` is set, then [`Applicablility::Suggested`] fixes are included.

    let fix_mode = if fix || fix_only {
        FixMode::Apply
    } else {
        FixMode::Generate
    };

    // At this point, we've assembled all our settings, and we're
    // ready to check the project

    let rules = ruleset(rule_selection, &preview_mode)?;

    let path_rules = rules_to_path_rules(&rules);
    let text_rules = rules_to_text_rules(&rules);
    let ast_entrypoints = ast_entrypoint_map(&rules);

    let files = get_files(files, file_extensions);
    let file_digits = files.len().to_string().len();
    let progress_bar_style = match progress_bar {
        ProgressBar::Fancy => {
            // Make progress bar with 60 char width, bright cyan colour (51)
            // Colours use some 8-bit representation
            let style_template = format!(
                "{{prefix}} {{pos:>{file_digits}}}/{{len}} [{{bar:60.51}}] [{{elapsed_precise}}]"
            );
            ProgressStyle::with_template(style_template.as_str())
                .unwrap()
                .progress_chars("━╸ ")
            // Alt: sub-character resolution "█▉▊▋▌▍▎▏  "
        }
        ProgressBar::Ascii => {
            // Same as fancy, but without colours and using basic characters
            let style_template = format!(
                "{{prefix}} {{pos:>{file_digits}}}/{{len}} [{{bar:60}}] [{{elapsed_precise}}]"
            );
            ProgressStyle::with_template(style_template.as_str())
                .unwrap()
                .progress_chars("=> ")
        }
        ProgressBar::Off => ProgressStyle::with_template("").unwrap(),
    };

    let diagnostics_per_file = files
        .par_iter()
        .progress_with_style(progress_bar_style)
        .with_prefix("Checking file:")
        .map(|path| {
            let filename = path.to_string_lossy();

            let source = match read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    let message = format!("Error opening file: {error}");
                    return Diagnostics::new(vec![DiagnosticMessage::from_error(
                        filename,
                        Diagnostic::new(IoError { message }, TextRange::default()),
                    )]);
                }
            };

            let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();

            match check_file(
                &path_rules,
                &text_rules,
                &ast_entrypoints,
                path,
                &file,
                &settings,
                fix_mode,
                unsafe_fixes,
            ) {
                Ok(violations) => violations,
                Err(msg) => {
                    let message = format!("Failed to process: {msg}");
                    Diagnostics::new(vec![DiagnosticMessage::from_error(
                        filename,
                        Diagnostic::new(IoError { message }, TextRange::default()),
                    )])
                }
            }
        });

    let mut all_diagnostics = diagnostics_per_file
        .fold(Diagnostics::default, |all_diagnostics, file_diagnostics| {
            all_diagnostics + file_diagnostics
        })
        .reduce(Diagnostics::default, |a, b| a + b);

    all_diagnostics.messages.par_sort_unstable();

    let total_errors = all_diagnostics.messages.len();

    let mut writer = Box::new(io::stdout());

    let mut printer_flags = PrinterFlags::empty();
    if !fix_only {
        printer_flags |= PrinterFlags::SHOW_VIOLATIONS;
    }
    if show_fixes {
        printer_flags |= PrinterFlags::SHOW_FIX_SUMMARY;
    }

    Printer::new(output_format, printer_flags, fix_mode, unsafe_fixes).write_once(
        files.len(),
        &all_diagnostics,
        &mut writer,
    )?;

    if total_errors == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::rule_selector::RuleSelector;

    use super::*;

    #[test]
    fn empty_select() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: None,
            extend_select: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = ruleset(args, &preview_mode)?;
        let preview = PreviewOptions::default();

        let all_rules: Vec<Rule> = DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect();

        assert_eq!(rules, all_rules);

        Ok(())
    }

    #[test]
    fn empty_select_with_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: None,
            extend_select: vec![],
        };

        let preview_mode = PreviewMode::Enabled;
        let rules = ruleset(args, &preview_mode)?;
        let preview = PreviewOptions {
            mode: preview_mode,
            require_explicit: false,
        };

        let all_rules: Vec<Rule> = DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect();

        assert_eq!(rules, all_rules);

        Ok(())
    }

    #[test]
    fn select_one_rule() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E000")?]),
            extend_select: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = ruleset(args, &preview_mode)?;
        let one_rules: Vec<Rule> = vec![Rule::IoError];

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn select_one_preview_rule_without_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E9904")?]),
            extend_select: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = ruleset(args, &preview_mode)?;
        let one_rules: Vec<Rule> = vec![];

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn select_one_preview_rule_with_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E9904")?]),
            extend_select: vec![],
        };

        let preview_mode = PreviewMode::Enabled;
        let rules = ruleset(args, &preview_mode)?;
        let one_rules: Vec<Rule> = vec![Rule::PreviewTestRule];

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn extend_select() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E000")?]),
            extend_select: vec![RuleSelector::from_str("E001")?],
        };

        let preview_mode = PreviewMode::default();
        let rules = ruleset(args, &preview_mode)?;
        let one_rules: Vec<Rule> = vec![Rule::IoError, Rule::SyntaxError];

        assert_eq!(rules, one_rules);

        Ok(())
    }

    use std::fs;

    use anyhow::{Context, Result};
    use tempfile::TempDir;
    use textwrap::dedent;

    #[test]
    fn find_and_check_fpm_toml() -> Result<()> {
        let tempdir = TempDir::new()?;
        let fpm_toml = tempdir.path().join("fpm.toml");
        fs::write(
            fpm_toml,
            dedent(
                r#"
                some-stuff = 1
                other-things = "hello"

                [extra.fortitude.check]
                ignore = ["T001"]
                "#,
            ),
        )?;

        let fpm = find_settings_toml(tempdir.path())?.context("Failed to find fpm.toml")?;
        let enabled = fortitude_enabled(fpm)?;
        assert!(enabled);

        Ok(())
    }
}
