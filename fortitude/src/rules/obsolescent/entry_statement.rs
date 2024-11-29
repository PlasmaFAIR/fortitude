use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
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
#[violation]
pub struct EntryStatement {}

impl Violation for EntryStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("entry statements are obsolescent, use module procedures with generic interface")
    }
}

impl AstRule for EntryStatement {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        some_vec![Diagnostic::from_node(EntryStatement {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["entry_statement"]
    }
}
