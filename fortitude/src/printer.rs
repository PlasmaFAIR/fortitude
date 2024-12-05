use std::cmp::Reverse;
use std::io::Write;

use anyhow::Result;
use bitflags::bitflags;
use colored::Colorize;
use itertools::{iterate, Itertools};

use crate::diagnostics::{Diagnostics, FixMap};
use crate::fs::relativize_path;
use crate::message::{
    AzureEmitter, Emitter, GithubEmitter, GitlabEmitter, GroupedEmitter, JsonEmitter,
    JsonLinesEmitter, JunitEmitter, PylintEmitter, RdjsonEmitter, SarifEmitter, TextEmitter,
};
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

pub(crate) struct Printer {
    format: OutputFormat,
    flags: Flags,
    fix_mode: FixMode,
    unsafe_fixes: UnsafeFixes,
}

impl Printer {
    pub(crate) fn new(
        format: OutputFormat,
        flags: Flags,
        fix_mode: FixMode,
        unsafe_fixes: UnsafeFixes,
    ) -> Self {
        Self {
            format,
            flags,
            fix_mode,
            unsafe_fixes,
        }
    }

    fn write_summary_text(
        &self,
        writer: &mut dyn Write,
        diagnostics: &Diagnostics,
        num_files: usize,
    ) -> Result<()> {
        // IO Errors indicate that we failed to read a file
        let num_failed = diagnostics
            .messages
            .iter()
            .filter(|x| x.name() == "IoError")
            .count();
        let total_files = num_files - num_failed;

        let fixables = FixableStatistics::try_from(diagnostics, self.unsafe_fixes);
        let fixed = diagnostics
            .fixed
            .values()
            .flat_map(std::collections::HashMap::values)
            .sum::<usize>();

        let file_no = if num_failed == 0 {
            format!(
                "fortitude: {} files scanned.",
                total_files.to_string().bold()
            )
        } else {
            format!(
                "fortitude: {} files scanned, {} could not be read.",
                total_files.to_string().bold(),
                num_failed.to_string().bold(),
            )
        };

        let remaining = diagnostics.messages.len();
        let total = fixed + remaining;

        let total_txt = total.to_string().bold();
        let fixed_txt = fixed.to_string().bold();
        let remaining_txt = remaining.to_string().bold();

        writeln!(writer, "{file_no}")?;

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

    pub(crate) fn write_once(
        &self,
        num_files: usize,
        diagnostics: &Diagnostics,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let fixables = FixableStatistics::try_from(diagnostics, self.unsafe_fixes);

        match self.format {
            OutputFormat::Concise | OutputFormat::Full => {
                TextEmitter::default()
                    .with_show_fix_status(true)
                    .with_show_fix_diff(self.flags.intersects(Flags::SHOW_FIX_DIFF))
                    .with_show_source(self.format == OutputFormat::Full)
                    .with_unsafe_fixes(crate::settings::UnsafeFixes::Hint)
                    .emit(writer, &diagnostics.messages)?;

                if self.flags.intersects(Flags::SHOW_FIX_SUMMARY) && !diagnostics.fixed.is_empty() {
                    writeln!(writer)?;
                    print_fix_summary(writer, &diagnostics.fixed)?;
                    writeln!(writer)?;
                }

                self.write_summary_text(writer, diagnostics, num_files)?;
            }
            OutputFormat::Github => {
                GithubEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Gitlab => {
                GitlabEmitter::default().emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Grouped => {
                GroupedEmitter::default()
                    .with_show_fix_status(show_fix_status(self.fix_mode, fixables.as_ref()))
                    .with_unsafe_fixes(self.unsafe_fixes)
                    .emit(writer, &diagnostics.messages)?;

                if self.flags.intersects(Flags::SHOW_FIX_SUMMARY) && !diagnostics.fixed.is_empty() {
                    writeln!(writer)?;
                    print_fix_summary(writer, &diagnostics.fixed)?;
                    writeln!(writer)?;
                }
                self.write_summary_text(writer, diagnostics, num_files)?;
            }
            OutputFormat::Json => {
                JsonEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Sarif => {
                SarifEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Azure => {
                AzureEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::JsonLines => {
                JsonLinesEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Rdjson => {
                RdjsonEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Junit => {
                JunitEmitter.emit(writer, &diagnostics.messages)?;
            }
            OutputFormat::Pylint => {
                PylintEmitter.emit(writer, &diagnostics.messages)?;
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
