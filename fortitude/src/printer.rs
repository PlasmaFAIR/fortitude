use std::{collections::BTreeSet, io::Write};

use anyhow::Result;
use colored::Colorize;

use crate::message::DiagnosticMessage;

pub(crate) struct Printer {}

impl Printer {
    pub(crate) fn new() -> Self {
        Self {}
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
        for d in diagnostics {
            writeln!(writer, "{d}")?;
        }

        self.write_summary_text(writer, diagnostics)?;

        Ok(())
    }
}
