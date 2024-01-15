use crate::active_rules::{get_all_rules, ActiveRules};
use crate::cli::CheckArgs;
use crate::parser::fortran_parser;
use crate::rules::{Method, Registry, Status};
use std::fs::read_to_string;
use std::path::PathBuf;

/// Parse a file, check it for issues, and return the report.
fn check_file(rules: &ActiveRules, path: &PathBuf) -> Result<(), String> {
    match read_to_string(path) {
        Ok(source) => {
            let mut parser = fortran_parser();
            if let Some(tree) = parser.parse(&source, None) {
                let root = tree.root_node();

                let mut violations = Vec::new();
                for (_, rule) in rules {
                    match rule.method {
                        Method::Tree(f) => violations.extend(f(rule.code, &root, &source)),
                        Method::File(f) => violations.extend(f(rule.code, &source)),
                        Method::Line(f) => {
                            for line in source.split('\n') {
                                violations.extend(f(rule.code, line))
                            }
                        }
                    }
                }

                violations.sort_unstable();
                if violations.is_empty() {
                    Ok(())
                } else {
                    let messages: Vec<String> = violations.iter().map(|x| x.to_string()).collect();
                    Err(messages.join("\n"))
                }
            } else {
                Err(String::from("Could not parse file."))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Get the list of active rules for this session.
fn get_rules<'a>(all_rules: &'a Registry, args: &'a CheckArgs) -> ActiveRules<'a> {
    // TODO update lists with settings file
    // TODO report error if rule does not exist
    if args.strict {
        ActiveRules::new(all_rules)
            .with_status(Status::Standard)
            .with_status(Status::Optional)
    } else if !args.select.is_empty() {
        ActiveRules::new(all_rules).add(&args.select)
    } else {
        ActiveRules::new(all_rules)
            .with_status(Status::Standard)
            .remove(&args.ignore)
            .add(&args.include)
    }
}

fn get_files_helper(path: PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            paths.extend(get_files_helper(entry.unwrap().path()));
        }
    } else if let Some(ext) = path.extension() {
        if ext == "f90" {
            paths.push(path);
        }
    }
    paths
}

/// Expand the input list of files to include all Fortran files.
fn get_files(files_in: &Vec<PathBuf>) -> Vec<PathBuf> {
    // TODO complain about files missing f90 extension
    // TODO Merge with helper function
    let mut paths = Vec::new();
    for path in files_in {
        paths.extend(get_files_helper(path.to_path_buf()));
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
            Ok(_) => {
                // File was okay, do nothing!
            }
            Err(s) => {
                errors.push(format!("Errors found for file: {}\n{}", file.display(), s));
            }
        }
    }
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("\n\n"))
    }
}
