use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

fn map_relational_symbols(name: &str) -> Option<&'static str> {
    match name {
        ".gt." => Some(">"),
        ".ge." => Some(">="),
        ".lt." => Some("<"),
        ".le." => Some("<="),
        ".eq." => Some("=="),
        ".ne." => Some("/="),
        _ => None,
    }
}

/// ## What does it do?
/// Checks for deprecated relational operators
///
/// ## Why is this bad?
/// Fortran 90 introduced the traditional symbols for relational operators: `>`,
/// `>=`, `<`, and so on. Prefer these over the deprecated forms `.gt.`, `.le.`, and
/// so on.
#[violation]
pub struct DeprecatedRelationalOperator {
    symbol: String,
    new_symbol: String,
}

impl Violation for DeprecatedRelationalOperator {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { symbol, new_symbol } = self;
        format!("deprecated relational operator '{symbol}', prefer '{new_symbol}' instead")
    }
}
impl AstRule for DeprecatedRelationalOperator {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let relation = node.child(1)?;
        let symbol = relation
            .to_text(src.source_text())?
            .to_lowercase()
            .to_string();
        let new_symbol = map_relational_symbols(symbol.as_str())?.to_string();
        some_vec![Diagnostic::from_node(
            Self { symbol, new_symbol },
            &relation
        )]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["relational_expression"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_relational_symbol() -> anyhow::Result<()> {
        let source = test_file(
            "
            program test
              if (0 .gt. 1) error stop
              if (1 .le. 0) error stop
              if (a.eq.b.and.a.ne.b) error stop
              if (1 == 2) error stop  ! OK
              if (2 /= 2) error stop  ! OK
            end program test
            ",
        );
        let expected: Vec<_> = [
            (2, 8, 2, 12, ".gt.", ">"),
            (3, 8, 3, 12, ".le.", "<="),
            (4, 7, 4, 11, ".eq.", "=="),
            (4, 18, 4, 22, ".ne.", "/="),
        ]
        .iter()
        .map(
            |(start_line, start_col, end_line, end_col, symbol, new_symbol)| {
                Diagnostic::from_start_end_line_col(
                    DeprecatedRelationalOperator {
                        symbol: symbol.to_string(),
                        new_symbol: new_symbol.to_string(),
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
        let actual = DeprecatedRelationalOperator::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
