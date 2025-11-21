use std::collections::BTreeSet;
use std::io::{self, BufWriter};
use std::process::ExitCode;

use anyhow::Result;
use colored::{Color, Colorize};
use fortitude_linter::registry::{Category, RuleNamespace};
use fortitude_linter::rule_selector::PreviewOptions;
use fortitude_linter::rules::{Rule, RuleGroup};
use fortitude_linter::settings::DEFAULT_SELECTORS;
use itertools::Itertools;
use ruff_diagnostics::FixAvailability;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use strum::IntoEnumIterator;
use termimad::{Alignment, MadSkin};
use textwrap::dedent;

use crate::cli::{ExplainCommand, HelpFormat};

#[derive(Serialize)]
struct Explanation<'a> {
    name: &'a str,
    code: String,
    category: &'a str,
    summary: &'a str,
    message_formats: &'a [&'a str],
    fix: String,
    #[expect(clippy::struct_field_names)]
    explanation: Option<&'a str>,
    preview: bool,
}

impl<'a> Explanation<'a> {
    fn from_rule(rule: &'a Rule) -> Self {
        let code = rule.noqa_code().to_string();
        let (category, _) = Category::parse_code(&code).unwrap();
        let fix = rule.fixable().to_string();
        Self {
            name: rule.as_ref(),
            code,
            category: category.name(),
            summary: rule.message_formats()[0],
            message_formats: rule.message_formats(),
            fix,
            explanation: rule.explanation(),
            preview: rule.is_preview(),
        }
    }
}

#[derive(Serialize)]
struct ShortExplanation<'a> {
    name: &'a str,
    code: String,
    summary: &'a str,
    fix: String,
    preview: bool,
}

impl<'a> ShortExplanation<'a> {
    fn from_rule(rule: &'a Rule) -> Self {
        let code = rule.noqa_code().to_string();
        let fix = rule.fixable().to_string();
        Self {
            name: rule.as_ref(),
            code,
            summary: rule.message_formats()[0],
            fix,
            preview: rule.is_preview(),
        }
    }
}

#[derive(Serialize)]
struct CategorySummary<'a> {
    name: &'a str,
    code: &'a str,
    description: &'a str,
}

impl<'a> CategorySummary<'a> {
    fn from_category(category: &'a Category) -> Self {
        Self {
            name: category.as_ref(),
            code: category.common_prefix(),
            description: category.description(),
        }
    }
}

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

/// Print shorter summary of rules
fn print_rule_list(rules: &[Rule]) {
    let mut output = String::new();

    let name_colour = if darkmode() {
        Color::White
    } else {
        Color::Black
    };

    for rule in rules {
        let name = rule.as_ref().color(name_colour).bold();
        let code = rule.noqa_code().to_string().bright_red().bold();
        let summary = rule.message_formats()[0];
        let status = match rule.group() {
            RuleGroup::Removed => "Rule has been removed".red(),
            RuleGroup::Deprecated => "Rule has been deprecated".red(),
            RuleGroup::Preview => "Rule is in preview".blue(),
            RuleGroup::Stable => "Rule is stable".green(),
        };
        let fixable_str = rule.fixable().to_string();
        let fixable = match rule.fixable() {
            FixAvailability::Always => fixable_str.green(),
            FixAvailability::Sometimes => fixable_str.green(),
            FixAvailability::None => fixable_str.normal(),
        };
        output.push_str(&format!("{code}\t{name}: {summary}. {status}. {fixable}\n"));
    }
    println!("{}", output);
}

fn print_rule_explanation(rules: &[Rule]) {
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

        // Replace links to the docs
        body = body.replace(
            "../settings.md",
            "https://fortitude.readthedocs.io/en/stable/settings",
        );

        let code = rule.noqa_code().to_string();
        let name = rule.as_ref();
        let title = format!("# {code}: {name}\n");
        outputs.push((title, dedent(body.as_str())));
    }
    outputs.sort_by(|a, b| {
        let ((a_code, _), (b_code, _)) = (a, b);
        a_code.cmp(b_code)
    });

    let skin = if colored::control::SHOULD_COLORIZE.should_colorize() {
        let mut skin = MadSkin::default();
        skin.headers[0].align = Alignment::Left;
        Some(skin)
    } else {
        None
    };

    let out_strings = outputs
        .iter()
        .map(|(code, desc)| format!("{code}\n{desc}"))
        .collect_vec();

    if let Some(skin) = &skin {
        skin.print_text(&out_strings.join("\n---\n"));
    } else {
        println!("{}", out_strings.join("\n"));
    }
}

fn print_categories() {
    let name_colour = if darkmode() {
        Color::White
    } else {
        Color::Black
    };

    for category in Category::iter() {
        println!(
            "{}\t{}: {}",
            category.common_prefix().bright_red().bold(),
            category.as_ref().color(name_colour).bold(),
            category.description()
        );
    }
}

/// Show rule names and explanations
pub fn explain(args: ExplainCommand) -> Result<ExitCode> {
    let rules = ruleset(&args)?;

    match args.output_format {
        HelpFormat::Text => {
            if args.list_categories {
                print_categories();
            } else if args.summary {
                print_rule_list(&rules);
            } else {
                print_rule_explanation(&rules);
            }
        }
        HelpFormat::Json => {
            let stdout = BufWriter::new(io::stdout().lock());
            let mut serialiser = serde_json::Serializer::pretty(stdout);
            let mut seq = serialiser.serialize_seq(None)?;
            if args.list_categories {
                for category in Category::iter() {
                    seq.serialize_element(&CategorySummary::from_category(&category))?;
                }
            } else if args.summary {
                for rule in rules {
                    seq.serialize_element(&ShortExplanation::from_rule(&rule))?;
                }
            } else {
                for rule in rules {
                    seq.serialize_element(&Explanation::from_rule(&rule))?;
                }
            }
            seq.end()?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
