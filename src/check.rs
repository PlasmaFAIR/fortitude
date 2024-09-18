use crate::ast::named_descendants_depth_first;
use crate::cli::CheckArgs;
use crate::rules::{default_ruleset, entrypoint_map, EntryPointMap, RuleSet};
use crate::settings::Settings;
use crate::violation;
use crate::{Category, Code, Diagnostic, Method, Violation};
use anyhow::Context;
use colored::Colorize;
use itertools::{chain, join};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;

/// Get the list of active rules for this session.
fn get_ruleset(args: &CheckArgs) -> RuleSet {
    // TODO update lists with settings file
    let mut ruleset = RuleSet::new();
    if !args.select.is_empty() {
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

fn tree_rules(
    entrypoints: &EntryPointMap,
    root: &Node,
    src: &str,
) -> anyhow::Result<Vec<(String, Violation)>> {
    let mut violations = Vec::new();
    for node in named_descendants_depth_first(root) {
        let empty = vec![];
        let rules = entrypoints.get(node.kind()).unwrap_or(&empty);
        for (code, rule) in rules {
            match rule.method() {
                Method::Tree(f) => {
                    if let Some(violation) = f(&node, src) {
                        violations.push((code.clone(), violation));
                    }
                }
                _ => {
                    anyhow::bail!("Non-Tree rule {} has incorrect entrypoints", code);
                }
            }
        }
    }
    Ok(violations)
}

/// Parse a file, check it for issues, and return the report.
fn check_file(
    entrypoints: &EntryPointMap,
    path: &Path,
    parser: &mut Parser,
    settings: &Settings,
) -> anyhow::Result<Vec<(String, Violation)>> {
    let mut violations = Vec::new();

    // Check path itself
    let empty_path_rules = vec![];
    let path_rules = entrypoints.get("PATH").unwrap_or(&empty_path_rules);
    for (code, rule) in path_rules {
        match rule.method() {
            Method::Path(f) => {
                if let Some(violation) = f(path) {
                    violations.push((code.clone(), violation));
                }
            }
            _ => {
                anyhow::bail!("Non-Path rule {} has incorrect entrypoints", code);
            }
        }
    }

    // Perform plain text analysis
    let empty_text_rules = vec![];
    let text_rules = entrypoints.get("TEXT").unwrap_or(&empty_text_rules);
    let source = read_to_string(path)?;
    for (code, rule) in text_rules {
        match rule.method() {
            Method::Text(f) => {
                violations.extend(
                    f(&source, settings)
                        .iter()
                        .map(|x| (code.clone(), x.clone())),
                );
            }
            _ => {
                anyhow::bail!("Non-Text rule {} has incorrect entrypoints", code);
            }
        }
    }

    // Perform AST analysis
    let tree = parser
        .parse(&source, None)
        .context("Could not parse file")?;
    let root = tree.root_node();
    violations.extend(tree_rules(entrypoints, &root, &source)?);

    Ok(violations)
}

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckArgs) -> i32 {
    let settings = Settings {
        line_length: args.line_length,
    };
    let ruleset = get_ruleset(&args);
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::language())
        .expect("Error loading Fortran grammar");
    match entrypoint_map(&ruleset) {
        Ok(entrypoints) => {
            let mut total_errors = 0;
            for file in get_files(&args.files) {
                match check_file(&entrypoints, &file, &mut parser, &settings) {
                    Ok(violations) => {
                        let mut diagnostics: Vec<Diagnostic> = violations
                            .into_iter()
                            .map(|(c, v)| Diagnostic::new(&file, c, &v))
                            .collect();
                        diagnostics.sort_unstable();
                        println!("{}", join(&diagnostics, "\n"));
                        total_errors += diagnostics.len();
                    }
                    Err(msg) => {
                        let err_code = Code::new(Category::Error, 0).to_string();
                        let err_msg = format!("Failed to process: {}", msg);
                        let violation = violation!(&err_msg);
                        let diagnostic = Diagnostic::new(&file, &err_code, &violation);
                        println!("{}", diagnostic);
                        total_errors += 1;
                    }
                }
            }
            if total_errors == 0 {
                0
            } else {
                let err_no = format!("Number of errors: {}", total_errors.to_string().bold());
                let info = "For more information, run:";
                let explain = format!("{} {}", "fortitude explain", "[ERROR_CODES]".bold());
                println!("\n-- {}\n-- {}\n\n    {}\n", err_no, info, explain);
                1
            }
        }
        Err(msg) => {
            eprintln!("{}: {}", "INTERNAL ERROR:".bright_red(), msg);
            1
        }
    }
}
