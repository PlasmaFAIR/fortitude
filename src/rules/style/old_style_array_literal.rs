use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

pub struct OldStyleArrayLiteral {}

impl Rule for OldStyleArrayLiteral {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        Fortran 2003 introduced a shorter syntax for array literals: `[...]`. While the
        older style, `(/.../)`, is still valid, the F2003 style is shorter and easier to
        match.
        "
    }
}

impl ASTRule for OldStyleArrayLiteral {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        if node.to_text(src.source_text())?.starts_with("(/") {
            let msg = "Array literal uses old-style syntax: prefer `[...]`";
            return some_vec!(Violation::from_node(msg, node));
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["array_literal"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
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
        let expected: Vec<Violation> = [
            (2, 20, 2, 31),
            (3, 20, 7, 4),
            (8, 17, 8, 28),
            (9, 10, 13, 4),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col)| {
            Violation::from_start_end_line_col(
                "Array literal uses old-style syntax: prefer `[...]`",
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = OldStyleArrayLiteral::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
