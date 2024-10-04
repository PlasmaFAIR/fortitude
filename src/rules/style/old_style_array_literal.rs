use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        if node.to_text(src)?.starts_with("(/") {
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
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_old_style_array_literal() -> anyhow::Result<()> {
        let source = dedent(
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
        let expected: Vec<Violation> = [(3, 21), (4, 21), (9, 18), (10, 11)]
            .iter()
            .map(|(line, col)| {
                let msg = "Array literal uses old-style syntax: prefer `[...]`";
                violation!(&msg, *line, *col)
            })
            .collect();
        let rule = OldStyleArrayLiteral::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
