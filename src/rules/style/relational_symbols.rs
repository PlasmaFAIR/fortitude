use std::collections::BTreeMap;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use lazy_static::lazy_static;
use tree_sitter::Node;

lazy_static! {
    static ref RELATIONAL_NAMES: BTreeMap<&'static str, &'static str> = {
        BTreeMap::from([
            (".gt.", ">"),
            (".ge.", ">="),
            (".lt.", "<"),
            (".le.", "<="),
            (".eq.", "=="),
            (".ne.", "/="),
        ])
    };
}

pub struct RelationalSymbol {}

impl Rule for RelationalSymbol {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        Fortran 90 introduced the traditional symbols for relational expressions: `>`,
        `>=`, `<`, and so on.
        "
    }
}

impl ASTRule for RelationalSymbol {
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        let relation = node.child(1)?;
        let symbol = relation.to_text(src)?.to_lowercase();
        let new_symbol = RELATIONAL_NAMES.get(symbol.as_str())?;
        let msg = format!("old style relational symbol '{symbol}', prefer '{new_symbol}'");
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
    fn test_old_style_array_literal() -> anyhow::Result<()> {
        let source = dedent(
            "
            program test
              if (0 .gt. 1) error stop
              if (1 .le. 0) error stop
              if (a.eq.b.and.a.ne.b) error stop
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
            let msg = format!("old style relational symbol '{symbol}', prefer '{new_symbol}'");
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = RelationalSymbol::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
