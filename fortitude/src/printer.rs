use std::{collections::BTreeSet, io::Write};

use anyhow::Result;
use bitflags::bitflags;
use colored::Colorize;

use crate::message::{DiagnosticMessage, Emitter, JsonEmitter, TextEmitter};
use crate::settings::OutputFormat;

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
}

impl Printer {
    pub(crate) fn new(format: OutputFormat, flags: Flags) -> Self {
        Self { format, flags }
    }

    fn write_summary_text(
        &self,
        writer: &mut dyn Write,
        diagnostics: &[DiagnosticMessage],
    ) -> Result<()> {
        let diagnostics_by_file: BTreeSet<_> = diagnostics.iter().map(|d| d.filename()).collect();
        let total_files = diagnostics_by_file.len();

        let file_no = format!(
            "fortitude: {} files scanned.",
            total_files.to_string().bold()
        );

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
        diagnostics: &[DiagnosticMessage],
        writer: &mut dyn Write,
    ) -> Result<()> {
        match self.format {
            OutputFormat::Concise | OutputFormat::Full => {
                TextEmitter::default()
                    .with_show_fix_status(true)
                    .with_show_fix_diff(self.flags.intersects(Flags::SHOW_FIX_DIFF))
                    .with_show_source(self.format == OutputFormat::Full)
                    .with_unsafe_fixes(crate::settings::UnsafeFixes::Hint)
                    .emit(writer, diagnostics)?;
                self.write_summary_text(writer, diagnostics)?;
            }
            OutputFormat::Json => {
                JsonEmitter.emit(writer, diagnostics)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
}
