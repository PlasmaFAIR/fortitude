use std::cmp::Reverse;
use std::fmt::Display;
use std::io::Write;

use anyhow::Result;
use bitflags::bitflags;
use colored::Colorize;
use itertools::{iterate, Itertools};
use serde::Serialize;

use crate::check::CheckResults;
use crate::diagnostics::{Diagnostics, FixMap};
use crate::fs::relativize_path;
use crate::logging::LogLevel;
use crate::message::{
    AzureEmitter, DiagnosticMessage, Emitter, GithubEmitter, GitlabEmitter, GroupedEmitter,
    JsonEmitter, JsonLinesEmitter, JunitEmitter, PylintEmitter, RdjsonEmitter, SarifEmitter,
    TextEmitter,
};
use crate::rules::Rule;
use crate::settings::{FixMode, OutputFormat, UnsafeFixes};

bitflags! {
    #[derive(Default, Debug, Copy, Clone)]
    pub(crate) struct Flags: u8 {
        /// Whether to show violations when emitting diagnostics.
        const SHOW_VIOLATIONS = 0b0000_0001;
        /// Whether to show a summary of the fixed violations when emitting diagnostics.
        const SHOW_FIX_SUMMARY = 0b0000_0100;
        /// Whether to show a diff of each fixed violation when emitting diagnostics.
        const SHOW_FIX_DIFF = 0b0000_1000;
    }
}

#[derive(Serialize)]
struct ExpandedStatistics {
    code: Option<SerializeRuleAsCode>,
    name: SerializeRuleAsTitle,
    count: usize,
    fixable: bool,
}

#[derive(Copy, Clone)]
struct SerializeRuleAsCode(Rule);

impl Serialize for SerializeRuleAsCode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.noqa_code().to_string())
    }
}

impl Display for SerializeRuleAsCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.noqa_code())
    }
}

impl From<Rule> for SerializeRuleAsCode {
    fn from(rule: Rule) -> Self {
        Self(rule)
    }
}

struct SerializeRuleAsTitle(Rule);

impl Serialize for SerializeRuleAsTitle {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl Display for SerializeRuleAsTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_ref())
    }
}

impl From<Option<Rule>> for SerializeRuleAsTitle {
    fn from(rule: Option<Rule>) -> Self {
        match rule {
            Some(rule) => Self(rule),
            // This is a bit weird, but it's because `DiagnosticMessage::rule`
            // returns `Option<Rule>`, leftover from ruff which treats
            // `SyntaxError` specially and we don't. Should just not return `Option` there
            None => Self(Rule::SyntaxError),
        }
    }
}

pub(crate) struct Printer {
    format: OutputFormat,
    log_level: LogLevel,
    flags: Flags,
    fix_mode: FixMode,
    unsafe_fixes: UnsafeFixes,
}

impl Printer {
    pub(crate) fn new(
        format: OutputFormat,
        log_level: LogLevel,
        flags: Flags,
        fix_mode: FixMode,
        unsafe_fixes: UnsafeFixes,
    ) -> Self {
        Self {
            format,
            log_level,
            flags,
            fix_mode,
            unsafe_fixes,
        }
    }

    fn write_summary_text(&self, writer: &mut dyn Write, results: &CheckResults) -> Result<()> {
        if self.log_level < LogLevel::Default {
            return Ok(());
        }

        let skipped = if results.files_skipped == 0 {
            "".to_string()
        } else {
            format!(
                ", {} could not be read",
                results.files_skipped.to_string().bold()
            )
        };

        let report = format!(
            "fortitude: {} files scanned{}.",
            results.files_checked.to_string().bold(),
            skipped
        );

        writeln!(writer, "{report}")?;

        let fixables = FixableStatistics::try_from(&results.diagnostics, self.unsafe_fixes);
        let fixed = results
            .diagnostics
            .fixed
            .values()
            .flat_map(std::collections::HashMap::values)
            .sum::<usize>();

        let remaining = results.diagnostics.messages.len();
        let total = fixed + remaining;

        let total_txt = total.to_string().bold();
        let fixed_txt = fixed.to_string().bold();
        let remaining_txt = remaining.to_string().bold();

        let explain = format!(
            "fortitude explain {},{},...",
            "X001".bold().bright_red(),
            "Y002".bold().bright_red()
        );
        let info = format!("For more information about specific rules, run:\n\n    {explain}\n");

        if fixed > 0 {
            writeln!(writer, "Number of errors: {total_txt} ({fixed_txt} fixed, {remaining_txt} remaining)\n\n{info}")?;
        } else if remaining > 0 {
            writeln!(writer, "Number of errors: {remaining_txt}\n\n{info}")?;
        } else {
            let success = "All checks passed!".bright_green();
            writeln!(writer, "{success}\n")?;
        }

        if let Some(fixables) = fixables {
            let fix_prefix = format!("[{}]", "*".cyan());

            if self.unsafe_fixes.is_hint() {
                if fixables.applicable > 0 && fixables.inapplicable_unsafe > 0 {
                    let es = if fixables.inapplicable_unsafe == 1 {
                        ""
                    } else {
                        "es"
                    };
                    writeln!(writer,
                                "{fix_prefix} {} fixable with the `--fix` option ({} hidden fix{es} can be enabled with the `--unsafe-fixes` option).",
                                fixables.applicable, fixables.inapplicable_unsafe
                            )?;
                } else if fixables.applicable > 0 {
                    // Only applicable fixes
                    writeln!(
                        writer,
                        "{fix_prefix} {} fixable with the `--fix` option.",
                        fixables.applicable,
                    )?;
                } else {
                    // Only inapplicable fixes
                    let es = if fixables.inapplicable_unsafe == 1 {
                        ""
                    } else {
                        "es"
                    };
                    writeln!(writer,
                                "No fixes available ({} hidden fix{es} can be enabled with the `--unsafe-fixes` option).",
                                fixables.inapplicable_unsafe
                            )?;
                }
            } else if fixables.applicable > 0 {
                writeln!(
                    writer,
                    "{fix_prefix} {} fixable with the --fix option.",
                    fixables.applicable
                )?;
            }
        }

        Ok(())
    }

    pub(crate) fn write_once(&self, results: &CheckResults, writer: &mut dyn Write) -> Result<()> {
        if matches!(self.log_level, LogLevel::Silent) {
            return Ok(());
        }

        let fixables = FixableStatistics::try_from(&results.diagnostics, self.unsafe_fixes);

        match self.format {
            OutputFormat::Concise | OutputFormat::Full => {
                TextEmitter::default()
                    .with_show_fix_status(true)
                    .with_show_fix_diff(self.flags.intersects(Flags::SHOW_FIX_DIFF))
                    .with_show_source(self.format == OutputFormat::Full)
                    .with_unsafe_fixes(crate::settings::UnsafeFixes::Hint)
                    .emit(writer, &results.diagnostics.messages)?;

                if self.flags.intersects(Flags::SHOW_FIX_SUMMARY)
                    && !results.diagnostics.fixed.is_empty()
                {
                    writeln!(writer)?;
                    print_fix_summary(writer, &results.diagnostics.fixed)?;
                    writeln!(writer)?;
                }

                self.write_summary_text(writer, results)?;
            }
            OutputFormat::Github => {
                GithubEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Gitlab => {
                GitlabEmitter::default().emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Grouped => {
                GroupedEmitter::default()
                    .with_show_fix_status(show_fix_status(self.fix_mode, fixables.as_ref()))
                    .with_unsafe_fixes(self.unsafe_fixes)
                    .emit(writer, &results.diagnostics.messages)?;

                if self.flags.intersects(Flags::SHOW_FIX_SUMMARY)
                    && !results.diagnostics.fixed.is_empty()
                {
                    writeln!(writer)?;
                    print_fix_summary(writer, &results.diagnostics.fixed)?;
                    writeln!(writer)?;
                }
                self.write_summary_text(writer, results)?;
            }
            OutputFormat::Json => {
                JsonEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Sarif => {
                SarifEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Azure => {
                AzureEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::JsonLines => {
                JsonLinesEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Rdjson => {
                RdjsonEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Junit => {
                JunitEmitter.emit(writer, &results.diagnostics.messages)?;
            }
            OutputFormat::Pylint => {
                PylintEmitter.emit(writer, &results.diagnostics.messages)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    pub(crate) fn write_statistics(
        &self,
        diagnostics: &Diagnostics,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let statistics: Vec<ExpandedStatistics> = diagnostics
            .messages
            .iter()
            .sorted_by_key(|message| (message.rule(), message.fixable()))
            .fold(
                vec![],
                |mut acc: Vec<(&DiagnosticMessage, usize)>, message| {
                    if let Some((prev_message, count)) = acc.last_mut() {
                        if prev_message.rule() == message.rule() {
                            *count += 1;
                            return acc;
                        }
                    }
                    acc.push((message, 1));
                    acc
                },
            )
            .iter()
            .map(|&(message, count)| ExpandedStatistics {
                code: message.rule().map(std::convert::Into::into),
                name: message.rule().into(),
                count,
                fixable: message.fixable(),
            })
            .sorted_by_key(|statistic| Reverse(statistic.count))
            .collect();

        if statistics.is_empty() {
            return Ok(());
        }

        match self.format {
            OutputFormat::Full | OutputFormat::Concise => {
                // Compute the maximum number of digits in the count and code, for all messages,
                // to enable pretty-printing.
                let count_width = num_digits(
                    statistics
                        .iter()
                        .map(|statistic| statistic.count)
                        .max()
                        .unwrap(),
                );
                let code_width = statistics
                    .iter()
                    .map(|statistic| {
                        statistic
                            .code
                            .map_or_else(String::new, |rule| rule.to_string())
                            .len()
                    })
                    .max()
                    .unwrap();
                let any_fixable = statistics.iter().any(|statistic| statistic.fixable);

                let fixable = format!("[{}] ", "*".cyan());
                let unfixable = "[ ] ";

                // By default, we mimic Flake8's `--statistics` format.
                for statistic in statistics {
                    writeln!(
                        writer,
                        "{:>count_width$}\t{:<code_width$}\t{}{}",
                        statistic.count.to_string().bold(),
                        statistic
                            .code
                            .map_or_else(String::new, |rule| rule.to_string())
                            .red()
                            .bold(),
                        if any_fixable {
                            if statistic.fixable {
                                &fixable
                            } else {
                                unfixable
                            }
                        } else {
                            ""
                        },
                        statistic.name,
                    )?;
                }

                if any_fixable {
                    writeln!(writer, "[*] fixable with `fortitude check --fix`",)?;
                }
                return Ok(());
            }
            OutputFormat::Json => {
                writeln!(writer, "{}", serde_json::to_string_pretty(&statistics)?)?;
            }
            _ => {
                anyhow::bail!(
                    "Unsupported serialization format for statistics: {:?}",
                    self.format
                )
            }
        }

        writer.flush()?;

        Ok(())
    }
}

fn num_digits(n: usize) -> usize {
    iterate(n, |&n| n / 10)
        .take_while(|&n| n > 0)
        .count()
        .max(1)
}

/// Return `true` if the [`Printer`] should indicate that a rule is fixable.
fn show_fix_status(fix_mode: FixMode, fixables: Option<&FixableStatistics>) -> bool {
    // If we're in application mode, avoid indicating that a rule is fixable.
    // If the specific violation were truly fixable, it would've been fixed in
    // this pass! (We're occasionally unable to determine whether a specific
    // violation is fixable without trying to fix it, so if fix is not
    // enabled, we may inadvertently indicate that a rule is fixable.)
    (!fix_mode.is_apply()) && fixables.is_some_and(FixableStatistics::any_applicable_fixes)
}

fn print_fix_summary(writer: &mut dyn Write, fixed: &FixMap) -> Result<()> {
    let total = fixed
        .values()
        .map(|table| table.values().sum::<usize>())
        .sum::<usize>();
    assert!(total > 0);
    let num_digits = num_digits(
        *fixed
            .values()
            .filter_map(|table| table.values().max())
            .max()
            .unwrap(),
    );

    let s = if total == 1 { "" } else { "s" };
    let label = format!("Fixed {total} error{s}:");
    writeln!(writer, "{}", label.bold().green())?;

    for (filename, table) in fixed
        .iter()
        .sorted_by_key(|(filename, ..)| filename.as_str())
    {
        writeln!(
            writer,
            "{} {}{}",
            "-".cyan(),
            relativize_path(filename).bold(),
            ":".cyan()
        )?;
        for (rule, count) in table.iter().sorted_by_key(|(.., count)| Reverse(*count)) {
            writeln!(
                writer,
                "    {count:>num_digits$} × {} ({})",
                rule.noqa_code().to_string().red().bold(),
                rule.as_ref(),
            )?;
        }
    }
    Ok(())
}

/// Statistics for [applicable][ruff_diagnostics::Applicability] fixes.
#[derive(Debug)]
struct FixableStatistics {
    applicable: u32,
    inapplicable_unsafe: u32,
}

impl FixableStatistics {
    fn try_from(diagnostics: &Diagnostics, unsafe_fixes: UnsafeFixes) -> Option<Self> {
        let mut applicable = 0;
        let mut inapplicable_unsafe = 0;

        for message in diagnostics.messages.iter() {
            if let Some(fix) = message.fix() {
                if fix.applies(unsafe_fixes.required_applicability()) {
                    applicable += 1;
                } else {
                    // Do not include inapplicable fixes at other levels that do not provide an opt-in
                    if fix.applicability().is_unsafe() {
                        inapplicable_unsafe += 1;
                    }
                }
            }
        }

        if applicable == 0 && inapplicable_unsafe == 0 {
            None
        } else {
            Some(Self {
                applicable,
                inapplicable_unsafe,
            })
        }
    }

    fn any_applicable_fixes(&self) -> bool {
        self.applicable > 0
    }
}
