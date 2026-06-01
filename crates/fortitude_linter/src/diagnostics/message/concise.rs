// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use crate::{
    diagnostics::{
        Diagnostic, DisplayDiagnosticConfig, Severity,
        stylesheet::{DiagnosticStylesheet, fmt_styled, fmt_with_hyperlink},
    },
    fs::relativize_path,
};

pub(super) struct ConciseRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> ConciseRenderer<'a> {
    pub(super) fn new(config: &'a DisplayDiagnosticConfig) -> Self {
        Self { config }
    }

    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        let stylesheet = if self.config.color {
            DiagnosticStylesheet::styled()
        } else {
            DiagnosticStylesheet::plain()
        };

        let sep = fmt_styled(":", stylesheet.separator);
        for diag in diagnostics {
            if let Some(span) = diag.primary_span() {
                write!(
                    f,
                    "{path}",
                    path = fmt_styled(relativize_path(span.file().name()), stylesheet.emphasis)
                )?;
                if let Some(range) = span.range() {
                    let diagnostic_source = span.file();
                    let start = diagnostic_source
                        .to_source_code()
                        .line_column(range.start());

                    write!(
                        f,
                        "{sep}{line}{sep}{col}",
                        line = start.line,
                        col = start.column,
                    )?;
                }
                write!(f, "{sep} ")?;
            }

            if self.config.hide_severity {
                if let Some(code) = diag.secondary_code() {
                    write!(
                        f,
                        "{code} ",
                        code = fmt_styled(
                            fmt_with_hyperlink(&code, diag.documentation_url(), &stylesheet),
                            stylesheet.secondary_code
                        )
                    )?;
                } else {
                    write!(
                        f,
                        "{id}: ",
                        id = fmt_styled(
                            fmt_with_hyperlink(
                                &diag.inner.id,
                                diag.documentation_url(),
                                &stylesheet
                            ),
                            stylesheet.secondary_code
                        )
                    )?;
                }
            } else {
                let (severity, severity_style) = match diag.severity() {
                    Severity::Info => ("info", stylesheet.info),
                    Severity::Warning => ("warning", stylesheet.warning),
                    Severity::Error => ("error", stylesheet.error),
                    Severity::Fatal => ("fatal", stylesheet.error),
                };
                write!(
                    f,
                    "{severity}[{id}] ",
                    severity = fmt_styled(severity, severity_style),
                    id = fmt_styled(
                        fmt_with_hyperlink(
                            &diag.secondary_code_or_id(),
                            diag.documentation_url(),
                            &stylesheet
                        ),
                        stylesheet.emphasis
                    )
                )?;
            }
            if self.config.show_fix_status {
                // Do not display an indicator for inapplicable fixes
                if diag.has_applicable_fix(self.config.fix_applicability()) {
                    write!(f, "[{fix}] ", fix = fmt_styled("*", stylesheet.separator))?;
                }
            }

            writeln!(f, "{message}", message = diag.concise_message())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ruff_diagnostics::Applicability;

    use crate::diagnostics::{
        OutputFormat,
        message::tests::{TestEnvironment, create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Concise);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        test.f90:6:5: error[S201] 'implicit none' set on the enclosing module
        test.f90:7:3: error[S061] end statement should read 'end subroutine foo'
        star_kind.f90:1:8: error[PORT021] integer*4 is non-standard, use integer(4)
        ");
    }

    #[test]
    fn show_fixes() {
        let (mut env, diagnostics) = create_diagnostics(OutputFormat::Concise);
        env.hide_severity(true);
        env.show_fix_status(true);
        env.fix_applicability(Applicability::DisplayOnly);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        test.f90:6:5: S201 [*] 'implicit none' set on the enclosing module
        test.f90:7:3: S061 [*] end statement should read 'end subroutine foo'
        star_kind.f90:1:8: PORT021 integer*4 is non-standard, use integer(4)
        ");
    }

    #[test]
    fn show_fixes_preview() {
        let (mut env, diagnostics) = create_diagnostics(OutputFormat::Concise);
        env.hide_severity(true);
        env.show_fix_status(true);
        env.fix_applicability(Applicability::DisplayOnly);
        env.preview(true);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        test.f90:6:5: S201 [*] 'implicit none' set on the enclosing module
        test.f90:7:3: S061 [*] end statement should read 'end subroutine foo'
        star_kind.f90:1:8: PORT021 integer*4 is non-standard, use integer(4)
        ");
    }

    #[test]
    fn show_fixes_syntax_errors() {
        let (mut env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Concise);
        env.hide_severity(true);
        env.show_fix_status(true);
        env.fix_applicability(Applicability::DisplayOnly);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @"
        syntax_errors.py:1:15: invalid-syntax: Expected one or more symbol names after import
        syntax_errors.py:3:12: invalid-syntax: Expected ')', found newline
        ");
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Concise);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @"
        syntax_errors.py:1:15: error[invalid-syntax] Expected one or more symbol names after import
        syntax_errors.py:3:12: error[invalid-syntax] Expected ')', found newline
        ");
    }

    #[test]
    fn missing_file() {
        let mut env = TestEnvironment::new();
        env.format(OutputFormat::Concise);

        let diag = env.err().build();

        insta::assert_snapshot!(
            env.render(&diag),
            @"error[stable-test-rule] main diagnostic message",
        );
    }
}
