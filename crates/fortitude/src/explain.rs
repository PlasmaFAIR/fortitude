use std::collections::BTreeSet;
use std::process::ExitCode;

use anyhow::Result;
use colored::{Color, Colorize};
use fortitude_linter::rule_selector::PreviewOptions;
use fortitude_linter::rules::{Rule, RuleGroup};
use fortitude_linter::settings::DEFAULT_SELECTORS;
use itertools::Itertools;
use ruff_diagnostics::FixAvailability;
use strum::IntoEnumIterator;
use textwrap::dedent;

use crate::cli::ExplainCommand;

/// Get the list of active rules for this session.
fn ruleset(args: &ExplainCommand) -> anyhow::Result<Vec<Rule>> {
    // TODO: Take this as an option
    let preview = PreviewOptions {
        mode: fortitude_linter::settings::PreviewMode::Enabled,
        require_explicit: false,
    };

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

fn darkmode() -> bool {
    if cfg!(test) || !colored::control::SHOULD_COLORIZE.should_colorize() {
        false
    } else {
        terminal_light::luma().unwrap_or(1.0) < 0.6
    }
}

/// Format list of all rules
fn format_rule_list() -> String {
    let mut output = String::new();

    let name_colour = if darkmode() {
        Color::White
    } else {
        Color::Black
    };

    output.push_str(&"All rules:".green());
    output.push('\n');
    for rule in Rule::iter() {
        let name = rule.as_ref().color(name_colour).bold();
        let code = rule.noqa_code().to_string().bright_red().bold();
        let status = match rule.group() {
            RuleGroup::Removed => "rule has been removed".red(),
            RuleGroup::Deprecated => "rule has been deprecated".red(),
            RuleGroup::Preview => "rule is in preview".blue(),
            RuleGroup::Stable => "rule is stable".green(),
        };
        let fixable = if matches!(
            rule.fixable(),
            FixAvailability::Always | FixAvailability::Sometimes
        ) {
            "fix available".green()
        } else {
            "fix not available".normal()
        };
        output.push_str(&format!("{code}\t{name}: {status}, {fixable}\n"));
    }
    output.push('\n');
    output
}

/// Show rule names and explanations
pub fn explain(args: ExplainCommand) -> Result<ExitCode> {
    // Just show the list of rules and exit
    if args.list {
        print!("{}", format_rule_list());
        return Ok(ExitCode::SUCCESS);
    }

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
        println!("{code}\n{desc}");
    }
    Ok(ExitCode::SUCCESS)
}
