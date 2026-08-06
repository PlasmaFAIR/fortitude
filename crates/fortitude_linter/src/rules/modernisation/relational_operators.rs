use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext, kind_ids};

use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

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
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedRelationalOperator {
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let relation = node.child(1)?;
        let symbol = relation.text().to_lowercase().to_string();
        let new_symbol = map_relational_symbols(symbol.as_str())?.to_string();

        let fix = Fix::safe_edit(relation.edit_replacement(new_symbol.clone()));

        some_vec![
            context
                .create_diagnostic(Self { symbol, new_symbol }, relation)
                .with_fix(fix)
        ]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["relational_expression"]
    }
}
