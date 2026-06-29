// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use std::ops::Deref;

use colored::Colorize;

use ruff_diagnostics::Applicability;
use ruff_source_file::{LineColumn, OneIndexed};

use crate::diagnostics::DisplayDiagnosticConfig;
use crate::fs::relativize_path;

use crate::Diagnostic;

pub struct GroupedRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> GroupedRenderer<'a> {
    pub(super) fn new(config: &'a DisplayDiagnosticConfig) -> Self {
        Self { config }
    }

    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        messages: &[Diagnostic],
    ) -> std::fmt::Result {
        for (filename, messages) in group_diagnostics_by_filename(messages) {
            // Compute the maximum number of digits in the row and column, for messages in
            // this file.

            let mut max_row_length = OneIndexed::MIN;
            let mut max_column_length = OneIndexed::MIN;

            for message in &messages {
                max_row_length = max_row_length.max(message.start_location.line);
                max_column_length = max_column_length.max(message.start_location.column);
            }

            let row_length = max_row_length.digits();
            let column_length = max_column_length.digits();

            // Print the filename.
            writeln!(f, "{}:", relativize_path(filename).underline())?;

            // Print each message.
            for message in messages {
                write!(
                    f,
                    "{}",
                    DisplayGroupedMessage {
                        message,
                        show_fix_status: self.config.show_fix_status(),
                        applicability: self.config.fix_applicability(),
                        row_length,
                        column_length,
                    }
                )?;
            }

            // Print a blank line between files
            writeln!(f)?;
        }

        Ok(())
    }
}

pub(super) struct DiagnosticWithLocation<'a> {
    pub diagnostic: &'a Diagnostic,
    pub start_location: LineColumn,
}

impl Deref for DiagnosticWithLocation<'_> {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        self.diagnostic
    }
}

pub(super) fn group_diagnostics_by_filename<'a>(
    diagnostics: &'a [Diagnostic],
) -> BTreeMap<String, Vec<DiagnosticWithLocation<'a>>> {
    let mut grouped_diagnostics = BTreeMap::default();
    for diagnostic in diagnostics {
        grouped_diagnostics
            .entry(diagnostic.expect_filename())
            .or_insert_with(Vec::new)
            .push(DiagnosticWithLocation {
                diagnostic,
                start_location: diagnostic.start_location().unwrap_or_default(),
            });
    }
    grouped_diagnostics
}

struct DisplayGroupedMessage<'a> {
    message: DiagnosticWithLocation<'a>,
    show_fix_status: bool,
    applicability: Applicability,
    row_length: NonZeroUsize,
    column_length: NonZeroUsize,
}

impl Display for DisplayGroupedMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let DiagnosticWithLocation {
            diagnostic,
            start_location,
        } = &self.message;

        write!(
            f,
            "  {row_padding}",
            row_padding = " ".repeat(self.row_length.get() - start_location.line.digits().get())
        )?;

        // Check if we're working on a jupyter notebook and translate positions with cell accordingly
        let (row, col) = (start_location.line, start_location.column);

        writeln!(
            f,
            "{row}{sep}{col}{col_padding} {code_and_body}",
            sep = ":".cyan(),
            col_padding =
                " ".repeat(self.column_length.get() - start_location.column.digits().get()),
            code_and_body = RuleCodeAndBody {
                diagnostic,
                show_fix_status: self.show_fix_status,
                applicability: self.applicability
            },
        )?;

        Ok(())
    }
}

pub(super) struct RuleCodeAndBody<'a> {
    pub(crate) diagnostic: &'a Diagnostic,
    pub(crate) show_fix_status: bool,
    pub(crate) applicability: Applicability,
}

impl Display for RuleCodeAndBody<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.show_fix_status
            && let Some(fix) = self.diagnostic.fix()
        {
            // Do not display an indicator for inapplicable fixes
            if fix.applies(self.applicability) {
                if let Some(code) = self.diagnostic.secondary_code() {
                    write!(f, "{} ", code.red().bold())?;
                }
                return write!(
                    f,
                    "{fix}{body}",
                    fix = format_args!("[{}] ", "*".cyan()),
                    body = self.diagnostic.concise_message(),
                );
            }
        };

        if let Some(code) = self.diagnostic.secondary_code() {
            write!(
                f,
                "{code} {body}",
                code = code.red().bold(),
                body = self.diagnostic.concise_message(),
            )
        } else {
            write!(
                f,
                "{code}: {body}",
                code = self.diagnostic.id().as_str().red().bold(),
                body = self.diagnostic.concise_message(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{
        OutputFormat,
        message::tests::{create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn default() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Grouped);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn fix_status() {
        let (mut env, diagnostics) = create_diagnostics(OutputFormat::Grouped);
        env.show_fix_status(true);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn fix_status_unsafe() {
        let (mut env, diagnostics) = create_diagnostics(OutputFormat::Grouped);
        env.fix_applicability(ruff_diagnostics::Applicability::Unsafe);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Grouped);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }
}
