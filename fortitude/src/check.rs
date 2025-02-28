use crate::allow_comments::{check_allow_comments, gather_allow_comments};
use crate::ast::FortitudeNode;
use crate::cli::{CheckArgs, GlobalConfigArgs};
use crate::configuration::{self, parse_config_file, Configuration};
use crate::diagnostics::{Diagnostics, FixMap};
use crate::fix::{fix_file, FixResult};
use crate::fs::get_files;
use crate::message::DiagnosticMessage;
use crate::printer::{Flags as PrinterFlags, Printer};
use crate::registry::AsRule;
use crate::rule_table::RuleTable;
#[cfg(any(feature = "test-rules", test))]
use crate::rules::testing::test_rules::{self, TestRule, TEST_RULES};
use crate::rules::Rule;
use crate::rules::{error::ioerror::IoError, AstRuleEnum, PathRuleEnum, TextRuleEnum};
use crate::settings::{self, CheckSettings, FixMode, ProgressBar, Settings};
use crate::show_files::show_files;
use crate::show_settings::show_settings;
use crate::stdin::read_from_stdin;
use crate::warn_user_once_by_message;
use crate::{fs, locator::Locator, warn_user_once};

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use itertools::Itertools;
use log::{debug, warn};
use rayon::prelude::*;
use ruff_diagnostics::Diagnostic;
use ruff_source_file::{SourceFile, SourceFileBuilder};
use ruff_text_size::TextRange;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufWriter};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use tree_sitter::{Parser, Tree};

/// Returns true if the command should read from standard input.
fn is_stdin(files: &[PathBuf], stdin_filename: Option<&Path>) -> bool {
    // If the user provided a `--stdin-filename`, always read from standard input.
    if stdin_filename.is_some() {
        if let Some(file) = files.iter().find(|file| file.as_path() != Path::new("-")) {
            warn_user_once!(
                "Ignoring file {} in favor of standard input.",
                file.display()
            );
        }
        return true;
    }

    let [file] = files else {
        return false;
    };
    // If the user provided exactly `-`, read from standard input.
    file == Path::new("-")
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_file(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Diagnostics> {
    let (mut messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
            ignore_allow_comments,
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
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                file,
                settings,
                ignore_allow_comments,
            )?;
            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
            ignore_allow_comments,
        )?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    // Ignore based on per-file-ignores.
    // If the DiagnosticMessage is discarded, its fix will also be ignored.
    let per_file_ignores = &settings.check.per_file_ignores;
    let per_file_ignores = if !messages.is_empty() && !per_file_ignores.is_empty() {
        fs::ignores_from_path(path, per_file_ignores)
    } else {
        vec![]
    };
    if !per_file_ignores.is_empty() {
        messages.retain(|message| {
            if let Some(rule) = message.rule() {
                !per_file_ignores.contains(&rule)
            } else {
                true
            }
        });
    }

    Ok(Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    })
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_only_file(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Vec<DiagnosticMessage>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;
    let tree = parser
        .parse(file.source_text(), None)
        .context("Failed to parse")?;

    let violations = check_path(
        rules,
        path_rules,
        text_rules,
        ast_entrypoints,
        path,
        file,
        settings,
        &tree,
        ignore_allow_comments,
    );

    Ok(violations
        .into_iter()
        .map(|v| DiagnosticMessage::from_ruff(file, v))
        .collect_vec())
}

/// Check an already parsed file. This actually does all the checking,
/// `check_only_file`/`check_and_fix_file` wrap this
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_path(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    tree: &Tree,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();
    let mut allow_comments = Vec::new();

    // Check file paths directly
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
    let root = tree.root_node();
    for node in once(root).chain(root.descendants()) {
        if let Some(rules) = ast_entrypoints.get(node.kind()) {
            for rule in rules {
                if let Some(violation) = rule.check(settings, &node, file) {
                    for v in violation {
                        violations.push(v);
                    }
                }
            }
        }
        if let Some(allow_rules) = gather_allow_comments(&node, file) {
            allow_comments.push(allow_rules);
        };
    }

    // Raise violations for internal test rules
    #[cfg(any(feature = "test-rules", test))]
    {
        for test_rule in TEST_RULES {
            if !rules.enabled(*test_rule) {
                continue;
            }
            let diagnostic = match test_rule {
                Rule::StableTestRule => test_rules::StableTestRule::check(),
                Rule::StableTestRuleSafeFix => test_rules::StableTestRuleSafeFix::check(),
                Rule::StableTestRuleUnsafeFix => test_rules::StableTestRuleUnsafeFix::check(),
                Rule::StableTestRuleDisplayOnlyFix => {
                    test_rules::StableTestRuleDisplayOnlyFix::check()
                }
                Rule::PreviewTestRule => test_rules::PreviewTestRule::check(),
                Rule::DeprecatedTestRule => test_rules::DeprecatedTestRule::check(),
                Rule::AnotherDeprecatedTestRule => test_rules::AnotherDeprecatedTestRule::check(),
                Rule::RemovedTestRule => test_rules::RemovedTestRule::check(),
                Rule::AnotherRemovedTestRule => test_rules::AnotherRemovedTestRule::check(),
                Rule::RedirectedToTestRule => test_rules::RedirectedToTestRule::check(),
                Rule::RedirectedFromTestRule => test_rules::RedirectedFromTestRule::check(),
                Rule::RedirectedFromPrefixTestRule => {
                    test_rules::RedirectedFromPrefixTestRule::check()
                }
                _ => unreachable!("All test rules must have an implementation"),
            };
            if let Some(diagnostic) = diagnostic {
                violations.push(diagnostic);
            }
        }
    }

    if (ignore_allow_comments.is_disabled() && !violations.is_empty())
        || rules.any_enabled(&[
            Rule::InvalidRuleCodeOrName,
            Rule::UnusedAllowComment,
            Rule::RedirectedAllowComment,
            Rule::DuplicatedAllowComment,
            Rule::DisabledAllowComment,
        ])
    {
        let ignored = check_allow_comments(&mut violations, &allow_comments, rules, file);
        if ignore_allow_comments.is_disabled() {
            for index in ignored.iter().rev() {
                violations.swap_remove(*index);
            }
        }
    }

    // Check violations for any remaining syntax errors. If any are found, discard violations
    // after it, as they may be false positives.
    if rules.enabled(Rule::SyntaxError) && root.has_error() {
        warn_user_once_by_message!(
            "Syntax errors detected in file: {}. Discarding subsequent violations from the AST.",
            path.to_string_lossy()
        );
        // Sort by byte-offset in the file
        violations.sort_by_key(|diagnostic| diagnostic.range.start());
        // Retain all violations up to the first syntax error, inclusive.
        // Text and path rules can be safely retained.
        let syntax_error_idx = violations
            .iter()
            .position(|diagnostic| diagnostic.kind.rule() == Rule::SyntaxError);
        if let Some(syntax_error_idx) = syntax_error_idx {
            violations = violations
                .into_iter()
                .enumerate()
                .filter_map(|(idx, diagnostic)| {
                    if idx <= syntax_error_idx || !diagnostic.kind.rule().is_ast_rule() {
                        Some(diagnostic)
                    } else {
                        None
                    }
                })
                .collect_vec();
        }
    }

    violations
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_and_fix_file<'a>(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &'a SourceFile,
    settings: &Settings,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<FixerResult<'a>> {
    let mut transformed = Cow::Borrowed(file);

    // Track the number of fixed errors across iterations.
    let mut fixed = FxHashMap::default();

    // As an escape hatch, bail after 100 iterations.
    let mut iterations = 0;

    // Track whether the _initial_ source code is valid syntax.
    let mut is_valid_syntax = false;

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;

    // Continuously fix until the source code stabilizes.
    loop {
        let tree = parser
            .parse(transformed.source_text(), None)
            .context("Failed to parse")?;

        // Map row and column locations to byte slices (lazily).
        let locator = Locator::new(transformed.source_text());

        let violations = check_path(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &transformed,
            settings,
            &tree,
            ignore_allow_comments,
        );

        if iterations == 0 {
            is_valid_syntax = !tree.root_node().has_error();
            if !is_valid_syntax {
                warn_user_once_by_message!(
                    "Syntax errors detected in file: {}. No fixes will be applied.",
                    path.to_string_lossy()
                );
                return Err(anyhow!(
                    "File contains syntax errors, no fixes will be applied"
                ));
            }
        } else if is_valid_syntax && tree.root_node().has_error() {
            report_fix_syntax_error(path, transformed.source_text(), fixed.keys().copied());
            return Err(anyhow!("Fix introduced a syntax error"));
        }

        // Apply fix
        if let Some(FixResult {
            code: fixed_contents,
            fixes: applied,
            ..
        }) = fix_file(
            &violations,
            &locator,
            settings.check.unsafe_fixes,
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

#[allow(clippy::print_stderr)]
fn report_fix_syntax_error(path: &Path, transformed: &str, rules: impl IntoIterator<Item = Rule>) {
    // TODO: include syntax error
    let codes = collect_rule_codes(rules);
    if cfg!(debug_assertions) {
        eprintln!(
            "{}{} Fix introduced a syntax error in `{}` with rule codes {codes}: \n---\n{transformed}\n---",
            "error".red().bold(),
            ":".bold(),
            fs::relativize_path(path),
        );
    } else {
        eprintln!(
            r#"
{}{} Fix introduced a syntax error. Reverting all changes.

This indicates a bug in Fortitude. If you could open an issue at:

    https://github.com/PlasmaFAIR/fortitude/issues/new?title=%5BFix%20error%5D

...quoting the contents of `{}`, the rule codes {}, along with the `fortitude.toml`/`fpm.toml` settings and executed command, we'd be very appreciative!
"#,
            "error".red().bold(),
            ":".bold(),
            fs::relativize_path(path),
            codes,
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

pub(crate) fn rules_to_path_rules(rules: &RuleTable) -> Vec<PathRuleEnum> {
    rules
        .iter_enabled()
        .filter_map(|rule| match TryFrom::try_from(rule) {
            Ok(path) => Some(path),
            _ => None,
        })
        .collect_vec()
}

pub(crate) fn rules_to_text_rules(rules: &RuleTable) -> Vec<TextRuleEnum> {
    rules
        .iter_enabled()
        .filter_map(|rule| match TryFrom::try_from(rule) {
            Ok(text) => Some(text),
            _ => None,
        })
        .collect_vec()
}

/// Create a mapping of AST entrypoints to lists of the rules and codes that operate on them.
pub(crate) fn ast_entrypoint_map<'a>(rules: &RuleTable) -> BTreeMap<&'a str, Vec<AstRuleEnum>> {
    let ast_rules: Vec<AstRuleEnum> = rules
        .iter_enabled()
        .filter_map(|rule| match TryFrom::try_from(rule) {
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

/// Helper object to store the results of all checks
pub(crate) struct CheckResults {
    /// All diagnostics found in all files
    pub(crate) diagnostics: Diagnostics,
    /// The number of files checked
    pub(crate) files_checked: usize,
    /// The number of files skipped
    pub(crate) files_skipped: usize,
}

impl CheckResults {
    fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            files_checked: 0,
            files_skipped: 0,
        }
    }

    fn from_stdin(diagnostics: Diagnostics) -> Self {
        Self {
            diagnostics,
            files_checked: 1,
            files_skipped: 0,
        }
    }

    fn add(mut self, status: CheckStatus) -> Self {
        match status {
            CheckStatus::Ok => self.files_checked += 1,
            CheckStatus::Violations(diagnostics) => {
                self.diagnostics += diagnostics;
                self.files_checked += 1;
            }
            CheckStatus::Skipped(diagnostics) => {
                self.diagnostics += diagnostics;
                self.files_skipped += 1;
            }
            CheckStatus::SkippedNoDiagnostic => self.files_skipped += 1,
        }
        self
    }

    fn merge(mut self, other: CheckResults) -> Self {
        self.diagnostics += other.diagnostics;
        self.files_checked += other.files_checked;
        self.files_skipped += other.files_skipped;
        self
    }

    fn sort(&mut self) {
        self.diagnostics.messages.par_sort_unstable();
    }
}

/// Enum used to report the result of a single file check
enum CheckStatus {
    /// The file was checked and no issues were found
    Ok,
    /// The file was checked and issues were found
    Violations(Diagnostics),
    /// The file was skipped due to an error
    Skipped(Diagnostics),
    /// The file was skipped but no violations were raised
    SkippedNoDiagnostic,
}

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckArgs, global_options: &GlobalConfigArgs) -> Result<ExitCode> {
    // First we need to find and read any config file
    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;
    let file_configuration = Configuration::from_options(
        parse_config_file(&global_options.config_file)?,
        &project_root,
    );

    // Now, we can override settings from the config file with options
    // from the CLI
    let settings = file_configuration.into_settings(&project_root, &args)?;

    let stdin_filename = args.stdin_filename;

    let mut writer: Box<dyn Write> = match args.output_file {
        Some(path) => {
            colored::control::set_override(false);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = File::create(path)?;
            Box::new(BufWriter::new(file))
        }
        _ => Box::new(BufWriter::new(io::stdout())),
    };
    let stderr_writer = Box::new(BufWriter::new(io::stderr()));

    let is_stdin = is_stdin(&args.files.unwrap_or_default(), stdin_filename.as_deref());

    if args.show_settings {
        show_settings(&settings, &mut writer)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args.show_files {
        show_files(&settings.file_resolver, is_stdin, &mut writer)?;
        return Ok(ExitCode::SUCCESS);
    }

    let CheckSettings {
        fix,
        fix_only,
        ref rules,
        unsafe_fixes,
        show_fixes,
        output_format,
        ignore_allow_comments,
        ..
    } = settings.check;

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

    let path_rules = rules_to_path_rules(rules);
    let text_rules = rules_to_text_rules(rules);
    let ast_entrypoints = ast_entrypoint_map(rules);

    let start = Instant::now();

    let files = get_files(&settings.file_resolver, is_stdin)?;
    debug!("Identified files to lint in: {:?}", start.elapsed());

    let results = if is_stdin {
        check_stdin(
            stdin_filename.map(fs::normalize_path).as_deref(),
            rules,
            &path_rules,
            &text_rules,
            &ast_entrypoints,
            &settings,
            fix_mode,
            ignore_allow_comments,
        )?
    } else {
        check_files(
            &files,
            rules,
            &path_rules,
            &text_rules,
            &ast_entrypoints,
            &settings,
            fix_mode,
            ignore_allow_comments,
        )?
    };

    // Always try to print violations (though the printer itself may suppress output)
    // If we're writing fixes via stdin, the transformed source code goes to the writer
    // so send the summary to stderr instead
    let mut summary_writer = if is_stdin && matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        stderr_writer
    } else {
        writer
    };

    let mut printer_flags = PrinterFlags::empty();
    if !fix_only {
        printer_flags |= PrinterFlags::SHOW_VIOLATIONS;
    }
    if show_fixes {
        printer_flags |= PrinterFlags::SHOW_FIX_SUMMARY;
    }

    let printer = Printer::new(
        output_format,
        global_options.log_level(),
        printer_flags,
        fix_mode,
        unsafe_fixes,
    );

    if args.statistics {
        printer.write_statistics(&results.diagnostics, &mut summary_writer)?;
    } else {
        printer.write_once(&results, &mut summary_writer)?;
    }

    let diagnostics = results.diagnostics;
    if !args.exit_zero {
        if fix_only {
            // If we're only fixing, we want to exit zero (since we've fixed all fixable
            // violations), unless we're explicitly asked to exit non-zero on fix.
            if args.exit_non_zero_on_fix && !diagnostics.fixed.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
        } else {
            // If we're running the linter (not just fixing), we want to exit non-zero if
            // there are any violations, unless we're explicitly asked to exit zero on
            // fix.
            if args.exit_non_zero_on_fix {
                if !diagnostics.fixed.is_empty() || !diagnostics.messages.is_empty() {
                    return Ok(ExitCode::FAILURE);
                }
            } else if !diagnostics.messages.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[allow(clippy::too_many_arguments)]
fn check_files(
    files: &[PathBuf],
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Result<CheckResults> {
    let file_digits = files.len().to_string().len();
    let progress_bar_style = match settings.check.progress_bar {
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

    let start = Instant::now();
    let mut results = files
        .par_iter()
        .progress_with_style(progress_bar_style)
        .with_prefix("Checking file:")
        .map(|path| {
            let filename = path.to_string_lossy();

            let source = match read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    if rules.enabled(Rule::IoError) {
                        let message = format!("Error opening file: {error}");
                        let diagnostics = vec![DiagnosticMessage::from_error(
                            filename,
                            Diagnostic::new(IoError { message }, TextRange::default()),
                        )];
                        return CheckStatus::Skipped(Diagnostics::new(diagnostics));
                    } else {
                        warn!(
                            "{}{}{} {error}",
                            "Error opening file ".bold(),
                            fs::relativize_path(path).bold(),
                            ":".bold()
                        );
                        return CheckStatus::SkippedNoDiagnostic;
                    }
                }
            };

            let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();

            match check_file(
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                &file,
                settings,
                fix_mode,
                ignore_allow_comments,
            ) {
                Ok(violations) => {
                    if violations.is_empty() {
                        CheckStatus::Ok
                    } else {
                        CheckStatus::Violations(violations)
                    }
                }
                Err(msg) => {
                    if rules.enabled(Rule::IoError) {
                        let message = format!("Failed to process: {msg}");
                        let diagnostics = vec![DiagnosticMessage::from_error(
                            filename,
                            Diagnostic::new(IoError { message }, TextRange::default()),
                        )];
                        CheckStatus::Skipped(Diagnostics::new(diagnostics))
                    } else {
                        warn!(
                            "{}{}{} {msg}",
                            "Failed to process ".bold(),
                            fs::relativize_path(path).bold(),
                            ":".bold()
                        );
                        CheckStatus::SkippedNoDiagnostic
                    }
                }
            }
        })
        .fold(CheckResults::new, |results, status| results.add(status))
        .reduce(CheckResults::new, |a, b| a.merge(b));

    results.sort();

    let duration = start.elapsed();
    debug!(
        "Checked {:?} files and skipped {:?} in: {:?}",
        results.files_checked, results.files_skipped, duration
    );

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn check_stdin(
    filename: Option<&Path>,
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Result<CheckResults> {
    let stdin = read_from_stdin()?;

    let path = filename.unwrap_or_else(|| Path::new("-"));
    let file = SourceFileBuilder::new(path.to_str().unwrap_or("-"), stdin.as_str()).finish();

    let (mut messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &file,
            settings,
            ignore_allow_comments,
        ) {
            if !fixed.is_empty() {
                match fix_mode {
                    FixMode::Apply => {
                        // Write the contents to stdout, regardless of whether any errors were fixed.
                        let out_file = &mut io::stdout().lock();
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
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                &file,
                settings,
                ignore_allow_comments,
            )?;
            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &file,
            settings,
            ignore_allow_comments,
        )?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    let per_file_ignores = &settings.check.per_file_ignores;
    // Ignore based on per-file-ignores.
    // If the DiagnosticMessage is discarded, its fix will also be ignored.
    let per_file_ignores = if !messages.is_empty() && !per_file_ignores.is_empty() {
        fs::ignores_from_path(path, per_file_ignores)
    } else {
        vec![]
    };
    if !per_file_ignores.is_empty() {
        messages.retain(|message| {
            if let Some(rule) = message.rule() {
                !per_file_ignores.contains(&rule)
            } else {
                true
            }
        });
    }

    let diagnostics = Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    };
    Ok(CheckResults::from_stdin(diagnostics))
}
