// Adapated from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

//! Generate Markdown documentation for applicable rules.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use regex::{Captures, Regex};
use strum::IntoEnumIterator;

use ruff_diagnostics::FixAvailability;

use fortitude::{
    options::Options,
    options_base::{OptionEntry, OptionsMetadata},
    registry::Rule,
};

use crate::{generate_rules_table, ROOT_DIR};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Write the generated docs to stdout (rather than to the filesystem).
    #[arg(long)]
    pub(crate) dry_run: bool,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    for rule in Rule::iter() {
        if let Some(explanation) = rule.explanation() {
            let mut output = String::new();

            output.push_str(&format!("# {} ({})", rule.as_ref(), rule.noqa_code()));
            output.push('\n');

            if rule.is_deprecated() {
                output.push_str(
                    r"**Warning: This rule is deprecated and will be removed in a future release.**",
                );
                output.push('\n');
                output.push('\n');
            }

            if rule.is_removed() {
                output.push_str(
                    r"**Warning: This rule has been removed and its documentation is only available for historical reasons.**",
                );
                output.push('\n');
                output.push('\n');
            }

            let fix_availability = rule.fixable();
            if matches!(
                fix_availability,
                FixAvailability::Always | FixAvailability::Sometimes
            ) {
                output.push_str(&fix_availability.to_string());
                output.push('\n');
                output.push('\n');
            }

            if rule.is_preview() {
                output.push_str(
                    r"This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.",
                );
                output.push('\n');
                output.push('\n');
            }

            process_documentation(
                explanation.trim(),
                &mut output,
                &rule.noqa_code().to_string(),
            );

            let filename = PathBuf::from(ROOT_DIR)
                .join("docs")
                .join("rules")
                .join(rule.as_ref())
                .with_extension("md");

            if args.dry_run {
                println!("{output}");
            } else {
                fs::create_dir_all("docs/rules").expect("make docs/rules dir");
                fs::write(filename, output).expect("write output");
            }
        }
    }

    let filename = PathBuf::from(ROOT_DIR).join("docs").join("rules.md");

    let rules_table = generate_rules_table::generate();
    fs::write(filename, rules_table).expect("Write rules table");

    Ok(())
}

fn process_documentation(documentation: &str, out: &mut String, rule_name: &str) {
    let mut in_options = false;
    let mut after = String::new();
    let mut referenced_options = HashSet::new();

    // HACK: This is an ugly regex hack that's necessary because mkdocs uses
    // a non-CommonMark-compliant Markdown parser, which doesn't support code
    // tags in link definitions
    // (see https://github.com/Python-Markdown/markdown/issues/280).
    let documentation = Regex::new(r"\[`([^`]*?)`]($|[^\[(])").unwrap().replace_all(
        documentation,
        |caps: &Captures| {
            format!(
                "[`{option}`][{option}]{sep}",
                option = &caps[1],
                sep = &caps[2]
            )
        },
    );

    for line in documentation.split_inclusive('\n') {
        if line.starts_with("## ") {
            in_options = line == "## Options\n";
        } else if in_options {
            if let Some(rest) = line.strip_prefix("- `") {
                let option = rest.trim_end().trim_end_matches('`');

                match Options::metadata().find(option) {
                    Some(OptionEntry::Field(field)) => {
                        if field.deprecated.is_some() {
                            eprintln!("Rule {rule_name} references deprecated option {option}.");
                        }
                    }
                    Some(_) => {}
                    None => {
                        panic!("Unknown option {option} referenced by rule {rule_name}");
                    }
                }

                let anchor = option.replace('.', "_");
                out.push_str(&format!("- [`{option}`][{option}]\n"));
                after.push_str(&format!("[{option}]: ../settings.md#{anchor}\n"));
                referenced_options.insert(option);

                continue;
            }
        }

        out.push_str(line);
    }

    let re = Regex::new(r"\[`([^`]*?)`]\[(.*?)]").unwrap();
    for (_, [option, _]) in re.captures_iter(&documentation).map(|c| c.extract()) {
        if let Some(OptionEntry::Field(field)) = Options::metadata().find(option) {
            if referenced_options.insert(option) {
                let anchor = option.replace('.', "_");
                after.push_str(&format!("[{option}]: ../settings.md#{anchor}\n"));
            }
            if field.deprecated.is_some() {
                eprintln!("Rule {rule_name} references deprecated option {option}.");
            }
        }
    }

    if !after.is_empty() {
        out.push('\n');
        out.push('\n');
        out.push_str(&after);
    }
}
