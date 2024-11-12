use std::process::ExitCode;

use crate::cli::ExplainArgs;
use crate::rules::{full_ruleset, Rule, RuleSet};
use anyhow::Result;
use colored::Colorize;
use ruff_diagnostics::FixAvailability;
use textwrap::dedent;

/// Get the list of active rules for this session.
fn ruleset(args: &ExplainArgs) -> anyhow::Result<Vec<Rule>> {
    let choices = if args.rules.is_empty() {
        full_ruleset()
    } else {
        let choices: RuleSet = args.rules.iter().map(|x| x.as_str()).collect();
        let diff: Vec<_> = choices.difference(&full_ruleset()).copied().collect();
        if !diff.is_empty() {
            anyhow::bail!("Unknown rule codes {:?}", diff);
        }
        choices
    };
    // unwrap ok here because we've already checked for valid codes
    let rules: Vec<_> = choices
        .iter()
        .map(|code| Rule::from_code(code).unwrap())
        .collect();
    Ok(rules)
}

/// Check all files, report issues found, and return error code.
pub fn explain(args: ExplainArgs) -> Result<ExitCode> {
    match ruleset(&args) {
        Ok(rules) => {
            let mut outputs = Vec::new();
            for rule in rules {
                let mut body = String::new();
                let fix_availability = rule.fixable();
                if matches!(
                    fix_availability,
                    FixAvailability::Always | FixAvailability::Sometimes
                ) {
                    body.push_str(&fix_availability.to_string());
                    body.push('\n');
                    body.push('\n');
                }
                if rule.is_preview() {
                    body.push_str(
                        r"This rule is in preview and is not stable. The `--preview` flag is required for use.",
                    );
                    body.push('\n');
                    body.push('\n');
                }

                if let Some(explanation) = rule.explanation() {
                    body.push_str(explanation);
                } else {
                    body.push_str("Message formats:");
                    for format in rule.message_formats() {
                        body.push('\n');
                        body.push_str(&format!("* {format}"));
                    }
                }

                let code = rule.noqa_code().suffix().to_string();
                let title = format!("# {code}");
                outputs.push((title.bright_red(), dedent(body.as_str())));
            }
            outputs.sort_by(|a, b| {
                let ((a_code, _), (b_code, _)) = (a, b);
                a_code.cmp(b_code)
            });
            for (code, desc) in outputs {
                println!("{}\n{}", code, desc);
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(msg) => {
            eprintln!("{}: {}", "ERROR".bright_red(), msg);
            Ok(ExitCode::FAILURE)
        }
    }
}
