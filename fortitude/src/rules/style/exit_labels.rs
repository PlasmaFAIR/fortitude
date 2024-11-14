use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// When using `exit` or `cycle` in a named `do` loop, the `exit`/`cycle` statement
/// should use the loop name
///
/// ## Example
/// ```fortran
/// name: do
///   exit name
/// end do name
/// ```
///
/// Using named loops is particularly useful for nested or complicated loops, as it
/// helps the reader keep track of the flow of logic. It's also the only way to `exit`
/// or `cycle` outer loops from within inner ones.
#[violation]
pub struct MissingExitOrCycleLabel {
    name: String,
    label: String,
}

impl Violation for MissingExitOrCycleLabel {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, label } = self;
        format!("'{name}' statement in named 'do' loop missing label '{label}'")
    }
}
impl AstRule for MissingExitOrCycleLabel {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        // Skip unlabelled loops
        let label = node
            .child_with_name("block_label_start_expression")?
            .to_text(src)?
            .trim_end_matches(':');

        let violations: Vec<Diagnostic> = node
            .named_descendants_except(["do_loop_statement"])
            .filter(|node| node.kind() == "keyword_statement")
            .map(|stmt| (stmt, stmt.to_text(src).unwrap_or_default().to_lowercase()))
            .filter(|(_, name)| name == "exit" || name == "cycle")
            .map(|(stmt, name)| {
                Diagnostic::from_node(
                    Self {
                        name: name.to_string(),
                        label: label.to_string(),
                    },
                    &stmt,
                )
            })
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["do_loop_statement"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_missing_exit_label() -> anyhow::Result<()> {
        let source = test_file(
            "
            program test
              label1: do
                if (.true.) then
                  EXIT
                end if
              end do label1

              label2: do
                if (.true.) exit
              end do label2

              label3: do
                exit label3
              end do label3

              label4: do
                if (.true.) exit label4
              end do label4

              label5: do i = 1, 2
                do j = 1, 2  ! unnamed inner loop: currently doesn't warn
                  cycle
                end do
              end do label5

              label6: do i = 1, 2
                inner: do j = 1, 2
                  if (.true.) CYCLE ! named inner loop: warns on inner loop
                end do inner
              end do label6

              label7: do
                cycle label7
              end do label7

              label8: do
                if (.true.) cycle label8
              end do label8

              do
                ! Don't warn on unnamed loops
                exit
              end do
            end program test
            ",
        );
        let expected: Vec<_> = [
            (4, 6, 4, 10, "exit", "label1"),
            (9, 16, 9, 20, "exit", "label2"),
            (28, 18, 28, 23, "cycle", "inner"),
        ]
        .iter()
        .map(
            |(start_line, start_col, end_line, end_col, stmt_kind, label)| {
                Diagnostic::from_start_end_line_col(
                    MissingExitOrCycleLabel {
                        name: stmt_kind.to_string(),
                        label: label.to_string(),
                    },
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            },
        )
        .collect();
        let actual = MissingExitOrCycleLabel::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
