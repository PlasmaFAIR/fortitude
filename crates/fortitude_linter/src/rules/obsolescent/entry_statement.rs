use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for `entry` statements.
///
/// ## Why is this bad?
/// `entry` statements are an obsolescent feature allowing more than entry point
/// into a procedure, enabling reuse of variables and executable
/// statements. However, they make the code much harder to follow and are prone
/// to bugs.
///
/// Multiple entry procedures can be replaced with modules to share data, and
/// private module procedures to reuse code.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[derive(ViolationMetadata)]
pub(crate) struct EntryStatement {}

impl Violation for EntryStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "entry statements are obsolescent, use module procedures with generic interface".to_string()
    }
}

impl AstRule for EntryStatement {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        some_vec![context.create_diagnostic(EntryStatement {}, node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["entry_statement"]
    }
}
