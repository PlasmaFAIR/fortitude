use std::io::Write;

use anyhow::Result;
use bitflags::bitflags;
use colored::Colorize;

use crate::message::{
    AzureEmitter, DiagnosticMessage, Emitter, GithubEmitter, GitlabEmitter, GroupedEmitter,
    JsonEmitter, JsonLinesEmitter, JunitEmitter, PylintEmitter, RdjsonEmitter, SarifEmitter,
    TextEmitter,
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
        diagnostics: &[DiagnosticMessage],
        num_files: usize,
    ) -> Result<()> {
        // IO Errors indicate that we failed to read a file
        let num_failed = diagnostics.iter().filter(|x| x.name() == "IoError").count();
        let total_files = num_files - num_failed;

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

        let total_errors = diagnostics.len();

        if total_errors == 0 {
            let success = "All checks passed!".bright_green();
            writeln!(writer, "\n{file_no}\n{success}\n")?;
        } else {
            let err_no = format!("Number of errors: {}", total_errors.to_string().bold());
            let info = "For more information about specific rules, run:";
            let explain = format!(
                "fortitude explain {},{},...",
                "X001".bold().bright_red(),
                "Y002".bold().bright_red()
            );
            writeln!(writer, "\n{file_no}\n{err_no}\n\n{info}\n\n    {explain}\n")?;
        }

        Ok(())
    }

    pub(crate) fn write_once(
        &self,
        num_files: usize,
        diagnostics: &[DiagnosticMessage],
        writer: &mut dyn Write,
    ) -> Result<()> {
        // TODO: implement tracking of fixables

        let fixables = FixableStatistics::try_from(diagnostics, self.unsafe_fixes);

        match self.format {
            OutputFormat::Concise | OutputFormat::Full => {
                TextEmitter::default()
                    .with_show_fix_status(true)
                    .with_show_fix_diff(self.flags.intersects(Flags::SHOW_FIX_DIFF))
                    .with_show_source(self.format == OutputFormat::Full)
                    .with_unsafe_fixes(crate::settings::UnsafeFixes::Hint)
                    .emit(writer, diagnostics)?;
                self.write_summary_text(writer, diagnostics, num_files)?;
            }
            OutputFormat::Github => {
                GithubEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Gitlab => {
                GitlabEmitter::default().emit(writer, diagnostics)?;
            }
            OutputFormat::Grouped => {
                GroupedEmitter::default()
                    .with_show_fix_status(show_fix_status(self.fix_mode, fixables.as_ref()))
                    .with_unsafe_fixes(self.unsafe_fixes)
                    .emit(writer, diagnostics)?;

                // if self.flags.intersects(Flags::SHOW_FIX_SUMMARY) {
                //     if !diagnostics.fixed.is_empty() {
                //         writeln!(writer)?;
                //         print_fix_summary(writer, &diagnostics.fixed)?;
                //         writeln!(writer)?;
                //     }
                // }
                self.write_summary_text(writer, diagnostics, num_files)?;
            }
            OutputFormat::Json => {
                JsonEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Sarif => {
                SarifEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Azure => {
                AzureEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::JsonLines => {
                JsonLinesEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Rdjson => {
                RdjsonEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Junit => {
                JunitEmitter.emit(writer, diagnostics)?;
            }
            OutputFormat::Pylint => {
                PylintEmitter.emit(writer, diagnostics)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
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

/// Statistics for [applicable][ruff_diagnostics::Applicability] fixes.
#[derive(Debug)]
struct FixableStatistics {
    applicable: u32,
    inapplicable_unsafe: u32,
}

impl FixableStatistics {
    fn try_from(diagnostics: &[DiagnosticMessage], unsafe_fixes: UnsafeFixes) -> Option<Self> {
        let mut applicable = 0;
        let mut inapplicable_unsafe = 0;

        for message in diagnostics.iter() {
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
