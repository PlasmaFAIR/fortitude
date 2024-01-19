use crate::cli::ExplainArgs;
use crate::rules::{full_ruleset, rulemap, RuleSet};
use crate::settings::Settings;
use colored::Colorize;
use textwrap::dedent;

/// Get the list of active rules for this session.
fn get_ruleset(args: &ExplainArgs) -> RuleSet {
    let mut ruleset = RuleSet::new();
    if args.rules.is_empty() {
        ruleset.extend(full_ruleset());
    } else {
        ruleset.extend(args.rules.iter().map(|x| x.to_string()));
    }
    ruleset
}

/// Check all files, report issues found, and return error code.
pub fn explain(args: ExplainArgs) -> i32 {
    let settings = Settings {
        strict: false,
        line_length: 100,
    };
    let ruleset = get_ruleset(&args);
    match rulemap(&ruleset, &settings) {
        Ok(rules) => {
            let mut outputs = Vec::new();
            for (code, rule) in &rules {
                outputs.push((
                    format!("{} {}", "#".bright_red(), code.bright_red()),
                    dedent(rule.explain()),
                ));
            }
            outputs.sort_by(|a, b| {
                let ((a_code, _), (b_code, _)) = (a, b);
                a_code.cmp(b_code)
            });
            for (code, desc) in outputs {
                println!("{}\n{}", code, desc);
            }
            0
        }
        Err(msg) => {
            eprintln!("{}: {}", "Error:".bright_red(), msg);
            1
        }
    }
}
