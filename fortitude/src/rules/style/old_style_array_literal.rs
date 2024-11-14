use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for old style array literals
///
/// ## Why is this bad?
/// Fortran 2003 introduced a shorter syntax for array literals: `[...]`. While the
/// older style, `(/.../)`, is still valid, the F2003 style is shorter and easier to
/// match.
#[violation]
pub struct OldStyleArrayLiteral {}

impl Violation for OldStyleArrayLiteral {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Array literal uses old-style syntax: prefer `[...]`")
    }
}
impl AstRule for OldStyleArrayLiteral {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.to_text(src.source_text())?.starts_with("(/") {
            return some_vec!(Diagnostic::from_node(Self {}, node));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["array_literal"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_old_style_array_literal() -> anyhow::Result<()> {
        let source = test_file(
            "
            program test
              integer :: a(3) = (/1, 2, 3/)
              integer :: b(3) = (/ &
                 1, &
                 2, &
                 3 &
              /)
             if (.true.) a = (/4, 5, 6/)
             b(1:3) = (/ &
                 4, &
                 5, &
                 6 &
              /)
             end program test
            ",
        );
        let expected: Vec<_> = [
            (2, 20, 2, 31),
            (3, 20, 7, 4),
            (8, 17, 8, 28),
            (9, 10, 13, 4),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col)| {
            Diagnostic::from_start_end_line_col(
                OldStyleArrayLiteral {},
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let actual = OldStyleArrayLiteral::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
