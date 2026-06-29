// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use ruff_source_file::SourceFile;
use ruff_text_size::TextRange;

use crate::{
    diagnostics::{Annotation, Diagnostic, Severity, SubDiagnosticSeverity},
    fs::relativize_path,
};

/// Generate error workflow command in GitHub Actions format.
/// See: [GitHub documentation](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message)
pub(super) struct GithubRenderer {}

impl GithubRenderer {
    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        for diagnostic in diagnostics {
            let severity = match diagnostic.severity() {
                Severity::Info => "notice",
                Severity::Warning => "warning",
                Severity::Error | Severity::Fatal => "error",
            };
            write!(
                f,
                "::{severity} title=fortitude ({code})",
                code = diagnostic.secondary_code_or_id()
            )?;

            if let Some(span) = diagnostic.primary_span() {
                let file = span.file();
                write!(f, ",file={file}", file = file.name())?;

                let (start_location, end_location) = {
                    let source_code = file.to_source_code();

                    span.range().map(|range| {
                        (
                            source_code.line_column(range.start()),
                            source_code.line_column(range.end()),
                        )
                    })
                }
                .unwrap_or_default();

                // GitHub Actions workflow commands have constraints on error annotations:
                // - `col` and `endColumn` cannot be set if `line` and `endLine` are different
                // See: https://github.com/astral-sh/ruff/issues/22074
                if start_location.line == end_location.line {
                    write!(
                        f,
                        ",line={row},col={column},endLine={end_row},endColumn={end_column}::",
                        row = start_location.line,
                        column = start_location.column,
                        end_row = end_location.line,
                        end_column = end_location.column,
                    )?;
                } else {
                    write!(
                        f,
                        ",line={row},endLine={end_row}::",
                        row = start_location.line,
                        end_row = end_location.line,
                    )?;
                }

                write!(
                    f,
                    "{path}:{row}:{column}: ",
                    path = relativize_path(file.name()),
                    row = start_location.line,
                    column = start_location.column,
                )?;
            } else {
                write!(f, "::")?;
            }

            if let Some(code) = diagnostic.secondary_code() {
                write!(f, "{code}")?;
            } else {
                write!(f, "{id}:", id = diagnostic.id())?;
            }

            write!(f, " {}", diagnostic.concise_message())?;

            // After rendering the main diagnostic, render its secondary annotations and
            // sub-diagnostics. Note that lines within a single diagnostic must be separated by
            // URL-encoded newlines (`%0A`) to render properly in GitHub annotations.
            for annotation in diagnostic
                .secondary_annotations()
                .filter_map(GithubAnnotation::from_annotation)
            {
                write!(f, "%0A{annotation}")?;
            }

            for subdiagnostic in diagnostic.sub_diagnostics() {
                let severity = match subdiagnostic.severity() {
                    SubDiagnosticSeverity::Help => "help",
                    SubDiagnosticSeverity::Info => "info",
                    SubDiagnosticSeverity::Warning => "warning",
                    SubDiagnosticSeverity::Error | SubDiagnosticSeverity::Fatal => "error",
                };
                if let Some(annotation) = subdiagnostic.primary_annotation()
                    && let span = annotation.get_span()
                    && let file = span.file()
                    && let Some(range) = span.range()
                {
                    let source_code = file.to_source_code();
                    let message = subdiagnostic.concise_message();
                    let start_location = source_code.line_column(range.start());
                    write!(
                        f,
                        "%0A  {path}:{row}:{column}: {severity}: {message}",
                        path = relativize_path(file.name()),
                        row = start_location.line,
                        column = start_location.column,
                    )?;
                } else {
                    write!(f, "%0A  {severity}: {}", subdiagnostic.concise_message())?;
                }

                for annotation in subdiagnostic
                    .secondary_annotations()
                    .filter_map(GithubAnnotation::from_annotation)
                {
                    write!(f, "%0A  {annotation}")?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

struct GithubAnnotation<'a> {
    message: &'a str,
    range: TextRange,
    file: &'a SourceFile,
}

impl<'a> GithubAnnotation<'a> {
    fn from_annotation(annotation: &'a Annotation) -> Option<Self> {
        let span = annotation.get_span();
        Some(Self {
            message: annotation.get_message()?,
            range: span.range()?,
            file: span.file(),
        })
    }
}

impl std::fmt::Display for GithubAnnotation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diagnostic_source = self.file;
        let source_code = diagnostic_source.to_source_code();
        let start_location = source_code.line_column(self.range.start());
        write!(
            f,
            "  {path}:{row}:{column}:",
            path = relativize_path(self.file.name()),
            row = start_location.line,
            column = start_location.column,
        )?;

        write!(f, " {message}", message = self.message)
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
        let (env, diagnostics) = create_diagnostics(OutputFormat::Github);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Github);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn missing_file() {
        let mut env = TestEnvironment::new();
        env.format(OutputFormat::Github);

        let diag = env.err().build();

        insta::assert_snapshot!(
            env.render(&diag),
            @"::error title=fortitude (stable-test-rule)::stable-test-rule: main diagnostic message",
        );
    }
}
