use std::collections::BTreeSet;
use std::process::ExitCode;

use crate::cli::ExplainArgs;
use crate::rule_selector::PreviewOptions;
use crate::rules::Rule;
use crate::settings::DEFAULT_SELECTORS;
use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use ruff_diagnostics::FixAvailability;
use textwrap::dedent;

/// Get the list of active rules for this session.
fn ruleset(args: &ExplainArgs) -> anyhow::Result<Vec<Rule>> {
    // TODO: Take this as an option
    let preview = PreviewOptions::default();

    // The rules_set keeps track of which rules have been selected.
    let mut rules_set: BTreeSet<Rule> = if args.rules.is_empty() {
        DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect()
    } else {
        BTreeSet::default()
    };

    for selector in args.rules.iter() {
        for rule in selector.rules(&preview) {
            rules_set.insert(rule);
        }
    }
    let rules = rules_set.into_iter().collect_vec();

    Ok(rules)
}

/// Check all files, report issues found, and return error code.
pub fn explain(args: ExplainArgs) -> Result<ExitCode> {
    let rules = ruleset(&args)?;

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

        let code = rule.noqa_code().to_string();
        let name = rule.as_ref();
        let title = format!("# {code}: {name}\n");
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
