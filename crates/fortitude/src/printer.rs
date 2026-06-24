use std::cmp::Reverse;
use std::io::Write;

use anyhow::Result;
use bitflags::bitflags;
use colored::Colorize;
use fortitude_linter::preview::{is_show_diff_enabled, is_warning_severity_enabled};
use itertools::{Itertools, iterate};
use serde::Serialize;

use crate::check::CheckResults;
use fortitude_linter::diagnostics::{
    Diagnostic, Diagnostics, DisplayDiagnosticConfig, FixMap, SecondaryCode, render_diagnostics,
};
use fortitude_linter::fs::relativize_path;
use fortitude_linter::logging::LogLevel;
use fortitude_linter::settings::{FixMode, OutputFormat, PreviewMode, UnsafeFixes};

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
struct ExpandedStatistics<'a> {
    code: Option<&'a SecondaryCode>,
    name: &'static str,
    count: usize,
    #[serde(rename = "fixable")]
    all_fixable: bool,
    fixable_count: usize,
}

impl ExpandedStatistics<'_> {
    fn any_fixable(&self) -> bool {
        self.fixable_count > 0
    }
}

/// Accumulator type for grouping diagnostics by code.
/// Format: (`code`, `representative_diagnostic`, `total_count`, `fixable_count`)
type DiagnosticGroup<'a> = (Option<&'a SecondaryCode>, &'a Diagnostic, usize, usize);

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

        let fixables = FixableStatistics::try_from(&results.diagnostics, self.unsafe_fixes);
        let fixed = results
            .diagnostics
            .fixed
            .values()
            .flat_map(std::collections::HashMap::values)
            .sum::<usize>();

        if self.flags.intersects(Flags::SHOW_VIOLATIONS) {
            let report = format!(
                "fortitude: {} files scanned{}.",
                results.files_checked.to_string().bold(),
                skipped
            );

            writeln!(writer, "{report}")?;

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
            let info =
                format!("For more information about specific rules, run:\n\n    {explain}\n");

            if fixed > 0 {
                writeln!(
                    writer,
                    "Number of errors: {total_txt} ({fixed_txt} fixed, {remaining_txt} remaining)\n\n{info}"
                )?;
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
                        writeln!(
                            writer,
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
                        writeln!(
                            writer,
                            "No fixes available ({} hidden fix{es} can be enabled with the `--unsafe-fixes` option).",
                            fixables.inapplicable_unsafe
                        )?;
                    }
                } else if fixables.applicable > 0 {
                    writeln!(
                        writer,
                        "{fix_prefix} {} fixable with the `--fix` option.",
                        fixables.applicable
                    )?;
                }
            }
        } else {
            // Unset SHOW_VIOLATIONS implies fix-only
            // Check if there are unapplied fixes
            let unapplied = {
                if let Some(fixables) = fixables {
                    fixables.inapplicable_unsafe
                } else {
                    0
                }
            };

            if unapplied > 0 {
                let es = if unapplied == 1 { "" } else { "es" };
                if fixed > 0 {
                    let s = if fixed == 1 { "" } else { "s" };
                    if self.fix_mode.is_apply() {
                        writeln!(
                            writer,
                            "Fixed {fixed} error{s} ({unapplied} additional fix{es} available with `--unsafe-fixes`)."
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "Would fix {fixed} error{s} ({unapplied} additional fix{es} available with `--unsafe-fixes`)."
                        )?;
                    }
                } else if self.fix_mode.is_apply() {
                    writeln!(
                        writer,
                        "No errors fixed ({unapplied} fix{es} available with `--unsafe-fixes`)."
                    )?;
                } else {
                    writeln!(
                        writer,
                        "No errors would be fixed ({unapplied} fix{es} available with `--unsafe-fixes`)."
                    )?;
                }
            } else if fixed > 0 {
                let s = if fixed == 1 { "" } else { "s" };
                if self.fix_mode.is_apply() {
                    writeln!(writer, "Fixed {fixed} error{s}.")?;
                } else {
                    writeln!(writer, "Would fix {fixed} error{s}.")?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn write_once(
        &self,
        results: &CheckResults,
        writer: &mut dyn Write,
        preview: PreviewMode,
    ) -> Result<()> {
        if matches!(self.log_level, LogLevel::Silent) {
            return Ok(());
        }

        if !self.flags.intersects(Flags::SHOW_VIOLATIONS) {
            if matches!(
                self.format,
                OutputFormat::Full | OutputFormat::Concise | OutputFormat::Grouped
            ) {
                if self.flags.intersects(Flags::SHOW_FIX_SUMMARY)
                    && !results.diagnostics.fixed.is_empty()
                {
                    writeln!(writer)?;
                    print_fix_summary(writer, &results.diagnostics.fixed)?;
                    writeln!(writer)?;
                }
                self.write_summary_text(writer, results)?;
            }
            return Ok(());
        }

        let diagnostics = &results.diagnostics;
        let fixables = FixableStatistics::try_from(diagnostics, self.unsafe_fixes);

        let config = DisplayDiagnosticConfig::new()
            .preview(preview.is_enabled())
            .hide_severity(!is_warning_severity_enabled(preview))
            .color(!cfg!(test) && colored::control::SHOULD_COLORIZE.should_colorize())
            .with_show_fix_status(show_fix_status(self.fix_mode, fixables.as_ref()))
            .with_fix_applicability(self.unsafe_fixes.required_applicability())
            .show_fix_diff(is_show_diff_enabled(preview));

        render_diagnostics(writer, self.format, config, &diagnostics.messages)?;

        if matches!(
            self.format,
            OutputFormat::Full | OutputFormat::Concise | OutputFormat::Grouped
        ) {
            if self.flags.intersects(Flags::SHOW_FIX_SUMMARY) && !diagnostics.fixed.is_empty() {
                writeln!(writer)?;
                print_fix_summary(writer, &diagnostics.fixed)?;
                writeln!(writer)?;
            }
            self.write_summary_text(writer, results)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub(crate) fn write_statistics(
        &self,
        diagnostics: &CheckResults,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let required_applicability = self.unsafe_fixes.required_applicability();
        let statistics: Vec<ExpandedStatistics> = diagnostics
            .diagnostics
            .messages
            .iter()
            .sorted_by_key(|diagnostic| diagnostic.secondary_code())
            .fold(vec![], |mut acc: Vec<DiagnosticGroup>, diagnostic| {
                let is_fixable = diagnostic
                    .fix()
                    .is_some_and(|fix| fix.applies(required_applicability));
                let code = diagnostic.secondary_code();

                if let Some((prev_code, _prev_message, count, fixable_count)) = acc.last_mut()
                    && *prev_code == code
                {
                    *count += 1;
                    if is_fixable {
                        *fixable_count += 1;
                    }
                    return acc;
                }
                acc.push((code, diagnostic, 1, usize::from(is_fixable)));
                acc
            })
            .iter()
            .map(
                |&(code, message, count, fixable_count)| ExpandedStatistics {
                    code,
                    name: message.name(),
                    count,
                    // Backward compatibility: `fixable` is true only when all violations are fixable.
                    // See: https://github.com/astral-sh/ruff/pull/21513
                    all_fixable: fixable_count == count,
                    fixable_count,
                },
            )
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
                    .map(|statistic| statistic.code.map_or(0, |s| s.len()))
                    .max()
                    .unwrap();
                let any_fixable = statistics.iter().any(ExpandedStatistics::any_fixable);

                let all_fixable = format!("[{}] ", "*".cyan());
                let partially_fixable = format!("[{}] ", "-".cyan());
                let unfixable = "[ ] ";

                // By default, we mimic Flake8's `--statistics` format.
                for statistic in &statistics {
                    writeln!(
                        writer,
                        "{:>count_width$}\t{:<code_width$}\t{}{}",
                        statistic.count.to_string().bold(),
                        statistic
                            .code
                            .map(SecondaryCode::as_str)
                            .unwrap_or_default()
                            .red()
                            .bold(),
                        if any_fixable {
                            if statistic.all_fixable {
                                &all_fixable
                            } else if statistic.any_fixable() {
                                &partially_fixable
                            } else {
                                unfixable
                            }
                        } else {
                            ""
                        },
                        statistic.name,
                    )?;
                }

                self.write_summary_text(writer, diagnostics)?;
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
