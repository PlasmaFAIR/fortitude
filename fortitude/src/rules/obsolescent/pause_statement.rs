use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Fix, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for `pause` statements.
///
/// ## Why is this bad?
/// `pause` statements were never properly standardised, doing different things
/// on different compilers, and were completely removed in Fortran 95. They can
/// usually be replaced with a simple call to `read(*,*)`
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[violation]
pub struct PauseStatement {}

impl Violation for PauseStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("`pause` statements are a deleted feature")
    }
    fn fix_title(&self) -> Option<String> {
        Some("Use 'read(*, *)' instead".into())
    }
}

impl AstRule for PauseStatement {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.child(0)?.to_text(src.source_text())?.to_lowercase() != "pause" {
            return None;
        }

        let fix = Fix::unsafe_edit(node.edit_replacement(src, "read(*, *)".to_string()));
        some_vec![Diagnostic::from_node(PauseStatement {}, node).with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["file_position_statement"]
    }
}
