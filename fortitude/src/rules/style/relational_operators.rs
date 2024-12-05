use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
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

impl AlwaysFixableViolation for DeprecatedRelationalOperator {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { symbol, new_symbol } = self;
        format!("deprecated relational operator '{symbol}', prefer '{new_symbol}' instead")
    }

    fn fix_title(&self) -> String {
        let Self { new_symbol, .. } = self;
        format!("Use '{new_symbol}'")
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

        let fix = Fix::safe_edit(relation.edit_replacement(new_symbol.clone()));

        some_vec![Diagnostic::from_node(Self { symbol, new_symbol }, &relation).with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["relational_expression"]
    }
}
