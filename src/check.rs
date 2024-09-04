use crate::cli::CheckArgs;
use crate::core::{Category, Code, Diagnostic, Method, Violation};
use crate::rules::{default_ruleset, rulemap, strict_ruleset, RuleBox, RuleSet};
use crate::settings::Settings;
use crate::violation;
use anyhow::Context;
use colored::Colorize;
use itertools::{chain, join};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Get the list of active rules for this session.
fn get_ruleset(args: &CheckArgs) -> RuleSet {
    // TODO update lists with settings file
    let mut ruleset = RuleSet::new();
    if args.strict {
        ruleset.extend(chain(&default_ruleset(), &strict_ruleset()).map(|x| x.to_string()));
    } else if !args.select.is_empty() {
        ruleset.extend(args.select.iter().map(|x| x.to_string()));
    } else {
        ruleset.extend(chain(&default_ruleset(), &args.include).map(|x| x.to_string()));
        for rule in &args.ignore {
            ruleset.remove(rule);
        }
    }
    ruleset
}

/// Helper function used with `filter` to select only paths that end in a Fortran extension.
/// Includes non-standard extensions, as these should be reported.
fn filter_fortran_extensions(path: &Path) -> bool {
    const FORTRAN_EXTS: &[&str] = &[
        "f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23",
    ];
    if let Some(ext) = path.extension() {
        // Can't use '&[&str].contains()', as extensions are of type OsStr
        FORTRAN_EXTS.iter().any(|&x| x == ext)
    } else {
        false
    }
}

/// Expand the input list of files to include all Fortran files.
fn get_files(files_in: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for path in files_in {
        if path.is_dir() {
            paths.extend(
                WalkDir::new(path)
                    .min_depth(1)
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .map(|x| x.path().to_path_buf())
                    .filter(|x| filter_fortran_extensions(x.as_path())),
            );
        } else {
            paths.push(path.to_path_buf());
        }
    }
    paths
}

/// Parse a file, check it for issues, and return the report.
fn check_file(rule: &RuleBox, path: &Path) -> anyhow::Result<Vec<Violation>> {
    let source = read_to_string(path)?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_fortran::language())
        .expect("Error loading Fortran grammar");
    let tree = parser
        .parse(&source, None)
        .context("Could not parse file")?;
    let root = tree.root_node();

    let mut violations = Vec::new();
    match rule.method() {
        Method::Path(f) => {
            if let Some(violation) = f(path) {
                violations.push(violation);
            }
        }
        Method::Tree(f) => {
            violations.extend(f(&root, &source));
        }
        Method::MultiLine(f) => {
            violations.extend(f(&source));
        }
        Method::Line(f) => {
            for (idx, line) in source.split('\n').enumerate() {
                if let Some(violation) = f(idx + 1, line) {
                    violations.push(violation);
                }
            }
        }
    }
    Ok(violations)
}

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckArgs) -> i32 {
    let settings = Settings {
        strict: args.strict,
        line_length: args.line_length,
    };
    let ruleset = get_ruleset(&args);
    match rulemap(&ruleset, &settings) {
        Ok(rules) => {
            let mut diagnostics = Vec::new();
            for file in get_files(&args.files) {
                for (code, rule) in &rules {
                    match check_file(rule, &file) {
                        Ok(violations) => {
                            diagnostics
                                .extend(violations.iter().map(|x| Diagnostic::new(&file, code, x)));
                        }
                        Err(msg) => {
                            let err_code = Code::new(Category::Error, 0).to_string();
                            let err_msg = format!("Failed to process: {}", msg);
                            let violation = violation!(&err_msg);
                            diagnostics.push(Diagnostic::new(&file, &err_code, &violation));
                        }
                    }
                }
            }
            if diagnostics.is_empty() {
                0
            } else {
                diagnostics.sort_unstable();
                println!("{}", join(&diagnostics, "\n"));
                println!("Number of errors: {}", diagnostics.len());
                1
            }
        }
        Err(msg) => {
            eprintln!("{}: {}", "Error:".bright_red(), msg);
            1
        }
    }
}
