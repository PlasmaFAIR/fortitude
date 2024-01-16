use crate::active_rules::{add_rules, get_all_rules, get_rules_with_status, remove_rules};
use crate::cli::CheckArgs;
use crate::parser::fortran_parser;
use crate::rules::{Method, Registry, Status};
use anyhow::Context;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Parse a file, check it for issues, and return the report.
fn check_file(rules: &Registry, path: &PathBuf) -> anyhow::Result<String> {
    let source = read_to_string(path)?;
    let mut parser = fortran_parser();
    let tree = parser
        .parse(&source, None)
        .context("Could not parse file")?;
    let root = tree.root_node();

    let mut violations = Vec::new();
    for rule in rules.values() {
        match rule.method() {
            Method::Tree(f) => violations.extend(f(rule.code(), &root, &source)),
            Method::File(f) => violations.extend(f(rule.code(), &source)),
            Method::Line(f) => {
                for line in source.split('\n') {
                    violations.extend(f(rule.code(), line))
                }
            }
        }
    }

    violations.sort_unstable();
    let messages: Vec<String> = violations.iter().map(|x| x.to_string()).collect();
    Ok(messages.join("\n"))
}

/// Get the list of active rules for this session.
fn get_rules(all_rules: &Registry, args: &CheckArgs) -> Registry {
    // TODO update lists with settings file
    // TODO report error if rule does not exist
    let mut active_rules = Registry::new();
    let standard_rules = get_rules_with_status(Status::Standard, all_rules);
    let optional_rules = get_rules_with_status(Status::Optional, all_rules);
    if args.strict {
        add_rules(all_rules, &mut active_rules, &standard_rules);
        add_rules(all_rules, &mut active_rules, &optional_rules);
    } else if !args.select.is_empty() {
        add_rules(all_rules, &mut active_rules, &args.select);
    } else {
        add_rules(all_rules, &mut active_rules, &standard_rules);
        remove_rules(&mut active_rules, &args.ignore);
        add_rules(all_rules, &mut active_rules, &args.include);
    }
    active_rules
}

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

pub fn check(args: CheckArgs) -> Option<String> {
    let all_rules = get_all_rules();
    let rules = get_rules(&all_rules, &args);
    let files = get_files(&args.files);
    let mut errors = Vec::new();
    for file in files {
        match check_file(&rules, &file) {
            Ok(s) => {
                if !s.is_empty() {
                    errors.push(format!("Issues found for file: {}\n{}", file.display(), s));
                }
            }
            Err(s) => {
                errors.push(format!("Failed to process file: {}\n{}", file.display(), s));
            }
        }
    }
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("\n\n"))
    }
}
