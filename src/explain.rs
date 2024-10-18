use std::process::ExitCode;

use crate::cli::ExplainArgs;
use crate::rules::{explain_rule, full_ruleset, RuleSet};
use crate::settings::default_settings;
use anyhow::Result;
use colored::Colorize;
use textwrap::dedent;

/// Get the list of active rules for this session.
fn ruleset(args: &ExplainArgs) -> anyhow::Result<RuleSet> {
    if args.rules.is_empty() {
        Ok(full_ruleset())
    } else {
        let choices: RuleSet = args.rules.iter().map(|x| x.as_str()).collect();
        let diff: Vec<_> = choices.difference(&full_ruleset()).copied().collect();
        if !diff.is_empty() {
            anyhow::bail!("Unknown rule codes {:?}", diff);
        }
        Ok(choices)
    }
}

/// Check all files, report issues found, and return error code.
pub fn explain(args: ExplainArgs) -> Result<ExitCode> {
    match ruleset(&args) {
        Ok(rules) => {
            let mut outputs = Vec::new();
            let settings = default_settings();
            for rule in rules {
                outputs.push((
                    format!("{} {}", "#".bright_red(), rule.bright_red()),
                    dedent(explain_rule(rule, &settings)),
                ));
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
