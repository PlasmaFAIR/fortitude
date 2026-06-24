// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use ruff_annotate_snippets::Renderer as AnnotateRenderer;

use super::Resolved;
use super::{Diagnostic, DisplayDiagnosticConfig};
use crate::diagnostics::{message::diff::Diff, stylesheet::DiagnosticStylesheet};

pub(super) struct FullRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> FullRenderer<'a> {
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

        let mut renderer = if self.config.color {
            AnnotateRenderer::styled()
        } else {
            AnnotateRenderer::plain()
        }
        .cut_indicator("…");

        renderer = renderer
            .error(stylesheet.error)
            .warning(stylesheet.warning)
            .info(stylesheet.info)
            .note(stylesheet.note)
            .help(stylesheet.help)
            .line_no(stylesheet.line_no)
            .emphasis(stylesheet.emphasis)
            .none(stylesheet.none)
            .hyperlink(stylesheet.hyperlink);

        for diag in diagnostics {
            let resolved = Resolved::new(diag, self.config);
            let renderable = resolved.to_renderable(self.config);
            for diag in renderable.diagnostics.iter() {
                writeln!(f, "{}", renderer.render(diag.to_annotate()))?;
            }

            if self.config.show_fix_diff
                && diag.has_applicable_fix(self.config.fix_applicability())
                && let Some(diff) = Diff::from_diagnostic(diag, &stylesheet)
            {
                write!(f, "{diff}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ruff_diagnostics::{Applicability, Edit, Fix};
    use ruff_text_size::{TextLen, TextRange, TextSize};

    use crate::{
        diagnostics::{
            Annotation, OutputFormat, Severity,
            message::tests::{
                TestEnvironment, create_diagnostics, create_syntax_error_diagnostics,
            },
        },
        rules::Rule,
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Full);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        error[S201]: 'implicit none' set on the enclosing module
         --> test.f90:6:5
          |
        4 | contains
        5 |   subroutine foo
        6 |     implicit none
          |     ^^^^^^^^^^^^^
        7 |   end subroutine
        8 | end module
          |
        help: Remove unnecessary 'implicit none'

        error[S061]: end statement should read 'end subroutine foo'
         --> test.f90:7:3
          |
        5 |   subroutine foo
        6 |     implicit none
        7 |   end subroutine
          |   ^^^^^^^^^^^^^^
        8 | end module
          |
        help: Write as 'end subroutine foo'
        info: Name from here
         --> test.f90:5:14
          |
        4 | contains
        5 |   subroutine foo
          |              ^^^ `foo` is defined here
        6 |     implicit none
        7 |   end subroutine
          |                 - insert name here
        8 | end module
          |

        error[PORT021]: integer*4 is non-standard, use integer(4)
         --> star_kind.f90:1:8
          |
        1 | integer*4 foo; end
          |        ^
          |
        ");
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Full);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        error[invalid-syntax]: Expected one or more symbol names after import
         --> syntax_errors.py:1:15
          |
        1 | from os import
          |               ^
        2 |
        3 | if call(foo
          |

        error[invalid-syntax]: Expected ')', found newline
         --> syntax_errors.py:3:12
          |
        1 | from os import
        2 |
        3 | if call(foo
          |            ^
        4 |     def bar():
        5 |         pass
          |
        ");
    }

    #[test]
    fn hide_severity_output() {
        let (mut env, diagnostics) = create_diagnostics(OutputFormat::Full);
        env.hide_severity(true);
        env.show_fix_status(true);
        env.fix_applicability(Applicability::DisplayOnly);

        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        S201 [*] 'implicit none' set on the enclosing module
         --> test.f90:6:5
          |
        4 | contains
        5 |   subroutine foo
        6 |     implicit none
          |     ^^^^^^^^^^^^^
        7 |   end subroutine
        8 | end module
          |
        help: Remove unnecessary 'implicit none'

        S061 [*] end statement should read 'end subroutine foo'
         --> test.f90:7:3
          |
        5 |   subroutine foo
        6 |     implicit none
        7 |   end subroutine
          |   ^^^^^^^^^^^^^^
        8 | end module
          |
        help: Write as 'end subroutine foo'
        info: Name from here
         --> test.f90:5:14
          |
        4 | contains
        5 |   subroutine foo
          |              ^^^ `foo` is defined here
        6 |     implicit none
        7 |   end subroutine
          |                 - insert name here
        8 | end module
          |

        PORT021 integer*4 is non-standard, use integer(4)
         --> star_kind.f90:1:8
          |
        1 | integer*4 foo; end
          |        ^
          |
        ");
    }

    #[test]
    fn hide_severity_syntax_errors() {
        let (mut env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Full);
        env.hide_severity(true);

        insta::assert_snapshot!(env.render_diagnostics(&diagnostics), @r"
        invalid-syntax: Expected one or more symbol names after import
         --> syntax_errors.py:1:15
          |
        1 | from os import
          |               ^
        2 |
        3 | if call(foo
          |

        invalid-syntax: Expected ')', found newline
         --> syntax_errors.py:3:12
          |
        1 | from os import
        2 |
        3 | if call(foo
          |            ^
        4 |     def bar():
        5 |         pass
          |
        ");
    }

    /// Check that the new `full` rendering code in `ruff_db` handles cases fixed by commit c9b99e4.
    ///
    /// For example, without the fix, we get diagnostics like this:
    ///
    /// ```
    /// error[no-indented-block]: Expected an indented block
    ///  --> example.py:3:1
    ///   |
    /// 2 | if False:
    ///   |          ^
    /// 3 | print()
    ///   |
    ///  ```
    ///
    /// where the caret points to the end of the previous line instead of the start of the next.
    #[test]
    fn empty_span_after_line_terminator() {
        let mut env = TestEnvironment::new();
        env.add(
            "example.py",
            r#"
if False:
print()
"#,
        );
        env.format(OutputFormat::Full);

        let diagnostic = env
            .builder(
                Rule::StableTestRule,
                Severity::Error,
                "Expected an indented block",
            )
            .primary("example.py", "3:0", "3:0", "")
            .build();

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: Expected an indented block
         --> example.py:3:1
          |
        2 | if False:
        3 | print()
          | ^
          |
        ");
    }

    /// Check that the new `full` rendering code in `ruff_db` handles cases fixed by commit 2922490.
    ///
    /// For example, without the fix, we get diagnostics like this:
    ///
    /// ```
    /// error[invalid-character-sub]: Invalid unescaped character SUB, use "\x1a" instead
    ///  --> example.py:1:25
    ///   |
    /// 1 | nested_fstrings = f'␈{f'{f'␛'}'}'
    ///   |                       ^
    ///   |
    ///  ```
    ///
    /// where the caret points to the `f` in the f-string instead of the start of the invalid
    /// character (`^Z`).
    #[test]
    fn unprintable_characters() {
        let mut env = TestEnvironment::new();
        env.add("example.py", "nested_fstrings = f'{f'{f''}'}'");
        env.format(OutputFormat::Full);

        let diagnostic = env
            .builder(
                Rule::StableTestRule,
                Severity::Error,
                r#"Invalid unescaped character SUB, use "\x1a" instead"#,
            )
            .primary("example.py", "1:24", "1:24", "")
            .build();

        insta::assert_snapshot!(env.render(&diagnostic), @r#"
        error[stable-test-rule]: Invalid unescaped character SUB, use "\x1a" instead
         --> example.py:1:25
          |
        1 | nested_fstrings = f'␈{f'{f'␛'}'}'
          |                         ^
          |
        "#);
    }

    #[test]
    fn multiple_unprintable_characters() -> std::io::Result<()> {
        let mut env = TestEnvironment::new();
        env.add("example.py", "");
        env.format(OutputFormat::Full);

        let diagnostic = env
            .builder(
                Rule::StableTestRule,
                Severity::Error,
                r#"Invalid unescaped character SUB, use "\x1a" instead"#,
            )
            .primary("example.py", "1:1", "1:1", "")
            .build();

        insta::assert_snapshot!(env.render(&diagnostic), @r#"
        error[stable-test-rule]: Invalid unescaped character SUB, use "\x1a" instead
         --> example.py:1:2
          |
        1 | ␈␛
          |  ^
          |
        "#);

        Ok(())
    }

    /// Ensure that the header column matches the column in the user's input, even if we've replaced
    /// tabs with spaces for rendering purposes.
    #[test]
    fn tab_replacement() {
        let mut env = TestEnvironment::new();
        env.add("example.py", "def foo():\n\treturn 1");
        env.format(OutputFormat::Full);

        let diagnostic = env.err().primary("example.py", "2:1", "2:9", "").build();

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
         --> example.py:2:2
          |
        1 | def foo():
        2 |     return 1
          |     ^^^^^^^^
          |
        ");
    }

    /// For file-level diagnostics, we expect to see the header line with the diagnostic information
    /// and the `-->` line with the file information but no lines of source code.
    #[test]
    fn file_level() {
        let mut env = TestEnvironment::new();
        env.add("example.py", "");
        env.format(OutputFormat::Full);

        let mut diagnostic = env.err().build();
        let span = env.path("example.py").with_range(TextRange::default());
        let mut annotation = Annotation::primary(span);
        annotation.hide_snippet(true);
        diagnostic.annotate(annotation);

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
        --> example.py:1:1
        ");
    }

    /// Carriage return (`\r`) is a valid line-ending in Python, so we should normalize this to a
    /// line feed (`\n`) for rendering. Otherwise we report a single long line for this case.
    #[test]
    fn normalize_carriage_return() {
        let mut env = TestEnvironment::new();
        env.add(
            "example.py",
            "# Keep parenthesis around preserved CR\rint(-\r    1)\rint(+\r    1)",
        );
        env.format(OutputFormat::Full);

        let mut diagnostic = env.err().build();
        let span = env
            .path("example.py")
            .with_range(TextRange::at(TextSize::new(39), TextSize::new(0)));
        let annotation = Annotation::primary(span);
        diagnostic.annotate(annotation);

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
         --> example.py:2:1
          |
        1 | # Keep parenthesis around preserved CR
        2 | int(-
          | ^
        3 |     1)
        4 | int(+
          |
        ");
    }

    /// Without stripping the BOM, we report an error in column 2, unlike Ruff.
    #[test]
    fn strip_bom() {
        let mut env = TestEnvironment::new();
        env.add("example.py", "\u{feff}import foo");
        env.format(OutputFormat::Full);

        let mut diagnostic = env.err().build();
        let span = env
            .path("example.py")
            .with_range(TextRange::at(TextSize::new(3), TextSize::new(0)));
        let annotation = Annotation::primary(span);
        diagnostic.annotate(annotation);

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
         --> example.py:1:1
          |
        1 | import foo
          | ^
          |
        ");
    }

    #[test]
    fn bom_with_default_range() {
        let mut env = TestEnvironment::new();
        env.add("example.py", "\u{feff}import foo");
        env.format(OutputFormat::Full);

        let mut diagnostic = env.err().build();
        let span = env.path("example.py").with_range(TextRange::default());
        let annotation = Annotation::primary(span);
        diagnostic.annotate(annotation);

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
         --> example.py:1:1
          |
        1 | import foo
          | ^
          |
        ");
    }

    /// We previously rendered this correctly, but the header was falling back to 1:1 for ranges
    /// pointing to the final newline in a file. Like Ruff, we now use the offset of the first
    /// character in the nonexistent final line in the header.
    #[test]
    fn end_of_file() {
        let mut env = TestEnvironment::new();
        let contents = "unexpected eof\n";
        env.add("example.py", contents);
        env.format(OutputFormat::Full);

        let mut diagnostic = env.err().build();
        let span = env
            .path("example.py")
            .with_range(TextRange::at(contents.text_len(), TextSize::new(0)));
        let annotation = Annotation::primary(span);
        diagnostic.annotate(annotation);

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule]: main diagnostic message
         --> example.py:2:1
          |
        1 | unexpected eof
          |               ^
          |
        ");
    }

    /// Test that we handle the width calculation for the line number correctly even for context
    /// lines at the end of a diff. For example, we want it to render like this:
    ///
    /// ```
    /// 8  |
    /// 9  |
    /// 10 |
    /// ```
    ///
    /// and not like this:
    ///
    /// ```
    /// 8 |
    /// 9 |
    /// 10 |
    /// ```
    #[test]
    fn longer_line_number_end_of_context() {
        let mut env = TestEnvironment::new();
        let contents = "\
line 1
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
line 10
        ";
        env.add("example.py", contents);
        env.format(OutputFormat::Full);
        env.show_fix_diff(true);
        env.show_fix_status(true);
        env.fix_applicability(Applicability::DisplayOnly);

        let mut diagnostic = env.err().primary("example.py", "3", "3", "label").build();
        diagnostic.help("Start of diff:");
        let target = "line 7";
        let line9 = contents.find(target).unwrap();
        let range = TextRange::at(TextSize::try_from(line9).unwrap(), target.text_len());
        diagnostic.set_fix(Fix::unsafe_edit(Edit::range_replacement(
            format!("fixed {target}"),
            range,
        )));

        insta::assert_snapshot!(env.render(&diagnostic), @r"
        error[stable-test-rule][*]: main diagnostic message
         --> example.py:3:1
          |
        1 | line 1
        2 | line 2
        3 | line 3
          | ^^^^^^ label
        4 | line 4
        5 | line 5
          |
        help: Start of diff:
        4  | line 4
        5  | line 5
        6  | line 6
           - line 7
        7  + fixed line 7
        8  | line 8
        9  | line 9
        10 | line 10
        note: This is an unsafe fix and may change runtime behavior
        ");
    }
}
