use crate::ast::FortitudeNode;
use crate::diagnostics::{Diagnostic, Fix, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kw};
use ruff_macros::derive_message_formats;
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
#[derive(ViolationMetadata)]
pub(crate) struct PauseStatement {}

impl Violation for PauseStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`pause` statements are a deleted feature".to_string()
    }
    fn fix_title(&self) -> Option<String> {
        Some("Use 'read(*, *)' instead".into())
    }
}

impl AstRule for PauseStatement {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.child(0)?.kind_id() != kw!("pause") {
            return None;
        }

        let fix = Fix::unsafe_edit(
            node.edit_replacement(context.source_file(), "read(*, *)".to_string()),
        );
        some_vec![
            context
                .create_diagnostic(PauseStatement {}, node)
                .with_fix(fix)
        ]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["file_position_statement"]
    }
}
