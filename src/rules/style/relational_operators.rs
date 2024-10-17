use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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

pub struct DeprecatedRelationalOperator {}

impl Rule for DeprecatedRelationalOperator {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        Fortran 90 introduced the traditional symbols for relational operators: `>`,
        `>=`, `<`, and so on. Prefer these over the deprecated forms `.gt.`, `.le.`, and
        so on.
        "
    }
}

impl ASTRule for DeprecatedRelationalOperator {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let relation = node.child(1)?;
        let symbol = relation.to_text(src.source_text())?.to_lowercase();
        let new_symbol = map_relational_symbols(symbol.as_str())?;
        let msg =
            format!("deprecated relational operator '{symbol}', prefer '{new_symbol}' instead");
        some_vec![Violation::from_node(msg, &relation)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["relational_expression"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
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
        let expected: Vec<Violation> =
            [
                (2, 8, 2, 12, ".gt.", ">"),
                (3, 8, 3, 12, ".le.", "<="),
                (4, 7, 4, 11, ".eq.", "=="),
                (4, 18, 4, 22, ".ne.", "/="),
            ]
            .iter()
            .map(
                |(start_line, start_col, end_line, end_col, symbol, new_symbol)| {
                    Violation::from_start_end_line_col(
                format!("deprecated relational operator '{symbol}', prefer '{new_symbol}' instead"),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
                },
            )
            .collect();
        let rule = DeprecatedRelationalOperator::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
