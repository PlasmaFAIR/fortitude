use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        let relation = node.child(1)?;
        let symbol = relation.to_text(src)?.to_lowercase();
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
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_relational_symbol() -> anyhow::Result<()> {
        let source = dedent(
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
        let expected: Vec<Violation> = [
            (3, 9, ".gt.", ">"),
            (4, 9, ".le.", "<="),
            (5, 8, ".eq.", "=="),
            (5, 19, ".ne.", "/="),
        ]
        .iter()
        .map(|(line, col, symbol, new_symbol)| {
            let msg =
                format!("deprecated relational operator '{symbol}', prefer '{new_symbol}' instead");
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = DeprecatedRelationalOperator::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
