use crate::ast::{parse, FortitudeNode};
use crate::cli::{CheckArgs, GlobalConfigArgs, FORTRAN_EXTS};
use crate::message::DiagnosticMessage;
use crate::printer::{Flags as PrinterFlags, Printer};
use crate::rule_selector::{PreviewOptions, RuleSelector, Specificity};
use crate::rules::Rule;
use crate::rules::{error::ioerror::IoError, AstRuleEnum, PathRuleEnum, TextRuleEnum};
use crate::settings::{OutputFormat, Settings, DEFAULT_SELECTORS};

use anyhow::{Context, Result};
use itertools::Itertools;
use rayon::prelude::*;
use ruff_diagnostics::Diagnostic;
use ruff_source_file::{SourceFile, SourceFileBuilder};
use ruff_text_size::TextRange;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use strum::IntoEnumIterator;
use toml::Table;
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
    pub output_format: OutputFormat,
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
            output_format: value.output_format.unwrap_or_default(),
        },
        None => CheckSettings::default(),
    };
    Ok(settings)
}

/// Get the list of active rules for this session.
fn ruleset(args: RuleSelection) -> anyhow::Result<Vec<Rule>> {
    // TODO: Take this as an option
    let preview = PreviewOptions::default();

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
pub(crate) fn check_file(
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
) -> anyhow::Result<Vec<Diagnostic>> {
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
    let tree = parse(file.source_text())?;
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

    Ok(violations)
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

    // At this point, we've assembled all our settings, and we're
    // ready to check the project

    let rules = ruleset(rule_selection)?;

    let path_rules = rules_to_path_rules(&rules);
    let text_rules = rules_to_text_rules(&rules);
    let ast_entrypoints = ast_entrypoint_map(&rules);

    let mut diagnostics: Vec<_> = get_files(files, file_extensions)
        .par_iter()
        .flat_map(|path| {
            let filename = path.to_string_lossy();

            let source = match read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    let message = format!("Error opening file: {error}");
                    return vec![DiagnosticMessage::from_error(
                        filename,
                        Diagnostic::new(IoError { message }, TextRange::default()),
                    )];
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
            ) {
                Ok(violations) => violations
                    .into_iter()
                    .map(|v| DiagnosticMessage::from_ruff(&file, v))
                    .collect_vec(),
                Err(msg) => {
                    let message = format!("Failed to process: {msg}");
                    vec![DiagnosticMessage::from_error(
                        filename,
                        Diagnostic::new(IoError { message }, TextRange::default()),
                    )]
                }
            }
        })
        .collect();

    diagnostics.par_sort_unstable();

    let total_errors = diagnostics.len();

    let mut writer = Box::new(io::stdout());

    let flags = PrinterFlags::SHOW_VIOLATIONS | PrinterFlags::SHOW_FIX_SUMMARY;

    Printer::new(output_format, flags).write_once(&diagnostics, &mut writer)?;

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

        let rules = ruleset(args)?;

        let preview = PreviewOptions::default();
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

        let rules = ruleset(args)?;
        let one_rules: Vec<Rule> = vec![Rule::IoError];

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

        let rules = ruleset(args)?;
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
