// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use crate::diagnostics::{Diagnostic, DisplayDiagnosticConfig, Severity};

/// Generate error logging commands for Azure Pipelines format.
/// See [documentation](https://learn.microsoft.com/en-us/azure/devops/pipelines/scripts/logging-commands?view=azure-devops&tabs=bash#logissue-log-an-error-or-warning)
pub(super) struct AzureRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> AzureRenderer<'a> {
    pub(super) fn new(config: &'a DisplayDiagnosticConfig) -> Self {
        Self { config }
    }

    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        for diag in diagnostics {
            let severity = match diag.severity() {
                Severity::None | Severity::Info => "debug",
                Severity::Warning => "warning",
                Severity::Error | Severity::Fatal => "error",
            };
            write!(f, "##vso[task.logissue type={severity};")?;
            if let Some(span) = diag.primary_span() {
                let filename = span.file().name();
                write!(f, "sourcepath={filename};")?;
                if let Some(range) = span.range() {
                    let location = span.file().to_source_code().line_column(range.start());
                    write!(
                        f,
                        "linenumber={line};columnnumber={col};",
                        line = location.line,
                        col = location.column,
                    )?;
                }
            }
            writeln!(
                f,
                "code={code};]{body}",
                code = self.config.format_rule_id(diag),
                body = diag.concise_message(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{
        OutputFormat,
        message::tests::{create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Azure);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Azure);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }
}
