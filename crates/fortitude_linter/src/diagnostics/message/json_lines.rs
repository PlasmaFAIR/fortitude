// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use crate::diagnostics::{Diagnostic, DisplayDiagnosticConfig, message::json::diagnostic_to_json};

pub(super) struct JsonLinesRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> JsonLinesRenderer<'a> {
    pub(super) fn new(config: &'a DisplayDiagnosticConfig) -> Self {
        Self { config }
    }
}

impl JsonLinesRenderer<'_> {
    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        for diag in diagnostics {
            writeln!(
                f,
                "{}",
                serde_json::json!(diagnostic_to_json(diag, self.config))
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
        let (env, diagnostics) = create_diagnostics(OutputFormat::JsonLines);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::JsonLines);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }
}
