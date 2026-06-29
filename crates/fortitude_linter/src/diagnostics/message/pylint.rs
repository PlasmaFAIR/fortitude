// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use crate::{
    diagnostics::{Diagnostic, SecondaryCode},
    fs::relativize_path,
};

/// Generate violations in Pylint format.
///
/// The format is given by this string:
///
/// ```python
/// "%(path)s:%(row)d: [%(code)s] %(text)s"
/// ```
///
/// See: [Flake8 documentation](https://flake8.pycqa.org/en/latest/internal/formatters.html#pylint-formatter)
pub(super) struct PylintRenderer {}

impl PylintRenderer {
    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        for diagnostic in diagnostics {
            let (filename, row) = diagnostic
                .primary_span_ref()
                .map(|span| {
                    let file = span.file();

                    let row = span
                        .range()
                        .map(|range| file.to_source_code().line_column(range.start()).line);

                    (relativize_path(file.name()), row)
                })
                .unwrap_or_default();

            let code = diagnostic
                .secondary_code()
                .map_or_else(|| diagnostic.name(), SecondaryCode::as_str);

            let row = row.unwrap_or_default();

            writeln!(
                f,
                "{path}:{row}: [{code}] {body}",
                path = filename,
                body = diagnostic.concise_message()
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{
        OutputFormat,
        message::tests::{TestEnvironment, create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Pylint);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Pylint);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn missing_file() {
        let mut env = TestEnvironment::new();
        env.format(OutputFormat::Pylint);

        let diag = env.err().build();

        insta::assert_snapshot!(
            env.render(&diag),
            @":1: [stable-test-rule] main diagnostic message",
        );
    }
}
