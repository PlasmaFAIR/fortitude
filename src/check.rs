use crate::ast::{parse, FortitudeNode};
use crate::cli::CheckArgs;
use crate::rules::{
    ast_entrypoint_map, default_ruleset, full_ruleset, path_rule_map, text_rule_map,
    ASTEntryPointMap, PathRuleMap, RuleSet, TextRuleMap,
};
use crate::settings::Settings;
use crate::violation;
use crate::{Diagnostic, Violation};
use anyhow::Result;
use colored::Colorize;
use itertools::{chain, join};
use ruff_source_file::{SourceFile, SourceFileBuilder};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use walkdir::WalkDir;

/// Get the list of active rules for this session.
fn ruleset(args: &CheckArgs) -> anyhow::Result<RuleSet> {
    // TODO Check that all rules in the set are valid, use Map::difference
    let mut choices = RuleSet::new();
    if !args.select.is_empty() {
        let select: RuleSet = args.select.iter().map(|x| x.as_str()).collect();
        choices.extend(select);
    } else {
        let include: RuleSet = args.include.iter().map(|x| x.as_str()).collect();
        let ignore: RuleSet = args.ignore.iter().map(|x| x.as_str()).collect();
        let defaults = default_ruleset();
        choices.extend(chain(&defaults, &include));
        for rule in &ignore {
            choices.remove(rule);
        }
    }
    let diff: Vec<_> = choices.difference(&full_ruleset()).copied().collect();
    if !diff.is_empty() {
        anyhow::bail!("Unknown rule codes {:?}", diff);
    }
    Ok(choices)
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
fn check_file(
    path_rules: &PathRuleMap,
    text_rules: &TextRuleMap,
    ast_entrypoints: &ASTEntryPointMap,
    path: &Path,
    file: &SourceFile,
) -> anyhow::Result<Vec<(String, Violation)>> {
    // TODO replace Vec<(String, Violation)> with Vec<(&str, Violation)>
    let mut violations = Vec::new();

    for (code, rule) in path_rules {
        if let Some(violation) = rule.check(path) {
            violations.push((code.to_string(), violation));
        }
    }

    // Perform plain text analysis
    for (code, rule) in text_rules {
        violations.extend(
            rule.check(file)
                .iter()
                .map(|x| (code.to_string(), x.clone())),
        );
    }

    // Perform AST analysis
    let tree = parse(file.source_text())?;
    for node in tree.root_node().named_descendants() {
        if let Some(rules) = ast_entrypoints.get(node.kind()) {
            for (code, rule) in rules {
                if let Some(violation) = rule.check(&node, file) {
                    for v in violation {
                        violations.push((code.to_string(), v));
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
fn read_to_string(path: &Path) -> std::io::Result<String> {
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

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckArgs) -> Result<ExitCode> {
    let settings = Settings {
        line_length: args.line_length,
    };
    match ruleset(&args) {
        Ok(rules) => {
            let path_rules = path_rule_map(&rules, &settings);
            let text_rules = text_rule_map(&rules, &settings);
            let ast_entrypoints = ast_entrypoint_map(&rules, &settings);
            let mut total_errors = 0;
            for path in get_files(&args.files, &args.file_extensions) {
                let filename = path.to_string_lossy();
                let empty_file = SourceFileBuilder::new(filename.as_ref(), "").finish();

                let source = match read_to_string(&path) {
                    Ok(source) => source,
                    Err(error) => {
                        let violation = violation!(format!("Error opening file: {error}"));
                        let diagnostic = Diagnostic::new(&empty_file, "E000", &violation);
                        println!("{diagnostic}");
                        total_errors += 1;
                        continue;
                    }
                };

                let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();

                match check_file(&path_rules, &text_rules, &ast_entrypoints, &path, &file) {
                    Ok(violations) => {
                        let mut diagnostics: Vec<Diagnostic> = violations
                            .into_iter()
                            .map(|(c, v)| Diagnostic::new(&file, c, &v))
                            .collect();
                        if !diagnostics.is_empty() {
                            diagnostics.sort_unstable();
                            println!("{}", join(&diagnostics, "\n"));
                        }
                        total_errors += diagnostics.len();
                    }
                    Err(msg) => {
                        let violation = violation!(format!("Failed to process: {msg}"));
                        let diagnostic = Diagnostic::new(&empty_file, "E000", &violation);
                        println!("{diagnostic}");
                        total_errors += 1;
                    }
                }
            }
            if total_errors == 0 {
                Ok(ExitCode::SUCCESS)
            } else {
                let err_no = format!("Number of errors: {}", total_errors.to_string().bold());
                let info = "For more information, run:";
                let explain = format!("{} {}", "fortitude explain", "[ERROR_CODES]".bold());
                println!("\n-- {}\n-- {}\n\n    {}\n", err_no, info, explain);
                Ok(ExitCode::FAILURE)
            }
        }
        Err(msg) => {
            eprintln!("{}: {}", "ERROR".bright_red(), msg);
            Ok(ExitCode::FAILURE)
        }
    }
}
