// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

//! Generate a Markdown-compatible table of supported lint rules.
//!
//! Used for <https://docs.astral.sh/ruff/rules/>.

use fortitude_linter::registry::{Category, Rule, RuleNamespace};
use fortitude_linter::rules::RuleGroup;

use std::borrow::Cow;
use strum::IntoEnumIterator;

use ruff_diagnostics::FixAvailability;

const FIX_SYMBOL: &str = "🛠️";
const PREVIEW_SYMBOL: &str = "🧪";
const REMOVED_SYMBOL: &str = "❌";
const WARNING_SYMBOL: &str = "⚠️";
const STABLE_SYMBOL: &str = "✔️";
const DEFAULT_SYMBOL: &str = "▶️";
const PEDANTIC_SYMBOL: &str = "⏸️";
const SPACER: &str = "&nbsp;&nbsp;&nbsp;&nbsp;";

fn generate_table(
    table_out: &mut String,
    rules: impl IntoIterator<Item = Rule>,
    linter: &Category,
) {
    table_out.push_str("| Code | Name | Message | |");
    table_out.push('\n');
    table_out.push_str("| ---- | ---- | ------- | ------: |");
    table_out.push('\n');
    for rule in rules {
        let status_token = match rule.group() {
            RuleGroup::Removed => {
                format!("<span title='Rule has been removed'>{REMOVED_SYMBOL}</span>")
            }
            RuleGroup::Deprecated => {
                format!("<span title='Rule has been deprecated'>{WARNING_SYMBOL}</span>")
            }
            #[allow(deprecated)]
            RuleGroup::Preview => {
                format!("<span title='Rule is in preview'>{PREVIEW_SYMBOL}</span>")
            }
            RuleGroup::Stable => {
                // A full opacity checkmark is a bit aggressive for indicating stable
                format!("<span title='Rule is stable' style='opacity: 0.6'>{STABLE_SYMBOL}</span>")
            }
        };

        let fix_token = match rule.fixable() {
            FixAvailability::Always | FixAvailability::Sometimes => {
                format!("<span title='Automatic fix available'>{FIX_SYMBOL}</span>")
            }
            FixAvailability::None => {
                format!(
                    "<span title='Automatic fix not available' style='opacity: 0.1' aria-hidden='true'>{FIX_SYMBOL}</span>"
                )
            }
        };

        let default_token = match rule.is_default() {
            true => {
                format!("<span title='Rule turned on by default'>{DEFAULT_SYMBOL}</span>")
            }
            false => {
                format!("<span title='Rule not on by default'>{PEDANTIC_SYMBOL}</span>")
            }
        };

        let tokens = format!("{status_token} {fix_token} {default_token}");

        let rule_name = rule.as_ref();

        // If the message ends in a bracketed expression (like: "Use {replacement}"), escape the
        // brackets. Otherwise, it'll be interpreted as an HTML attribute via the `attr_list`
        // plugin. (Above, we'd convert to "Use {replacement\}".)
        let message = rule.message_formats()[0];
        let message = if let Some(prefix) = message.strip_suffix('}') {
            Cow::Owned(format!("{prefix}\\}}"))
        } else {
            Cow::Borrowed(message)
        };

        // Start and end of style spans
        let mut ss = "";
        let mut se = "";
        if rule.is_removed() {
            ss = "<span style='opacity: 0.5', title='This rule has been removed'>";
            se = "</span>";
        } else if rule.is_deprecated() {
            ss = "<span style='opacity: 0.8', title='This rule has been deprecated'>";
            se = "</span>";
        }

        // TODO: Add back anchor when building proper docs
        #[allow(clippy::or_fun_call)]
        table_out.push_str(&format!(
            "| {ss}{0}{1}{se} | {ss}{2}{se} | {ss}{3}{se} | {ss}{4}{se} |",
            linter.common_prefix(),
            linter.code_for_rule(rule).unwrap(),
            rule.explanation()
                .is_some()
                .then_some(format_args!("[{rule_name}](rules/{rule_name}.md)"))
                .unwrap_or(format_args!("{rule_name}")),
            message,
            tokens,
        ));
        table_out.push('\n');
    }
    table_out.push('\n');
}

pub(crate) fn generate() -> String {
    // Generate the table string.
    let mut table_out = String::new();

    table_out.push_str("# Rules");
    table_out.push('\n');
    table_out.push('\n');

    table_out.push_str("### Legend");
    table_out.push('\n');

    table_out.push_str(&format!(
        "{SPACER}{STABLE_SYMBOL}{SPACER} The rule is stable."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{PREVIEW_SYMBOL}{SPACER} The rule is unstable and is in [\"preview\"](faq.md#what-is-preview)."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{WARNING_SYMBOL}{SPACER} The rule has been deprecated and will be removed in a future release."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{REMOVED_SYMBOL}{SPACER} The rule has been removed only the documentation is available."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{FIX_SYMBOL}{SPACER} The rule is automatically fixable by the `--fix` command-line option."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{DEFAULT_SYMBOL}{SPACER} The rule is turned on by default."
    ));
    table_out.push_str("<br />");

    table_out.push_str(&format!(
        "{SPACER}{PEDANTIC_SYMBOL}{SPACER} The rule is turned off by default."
    ));
    table_out.push_str("<br />");

    table_out.push('\n');

    for linter in Category::iter() {
        let codes_csv = linter.common_prefix().to_string();
        table_out.push_str(&format!("### {} ({codes_csv})", linter.name()));
        table_out.push('\n');
        table_out.push('\n');

        // TODO: cross-link to options

        generate_table(&mut table_out, linter.all_rules(), &linter);
    }

    // Ensure only one newline at the end
    let mut table_out = table_out.trim_end().to_string();
    table_out.push('\n');

    table_out
}
