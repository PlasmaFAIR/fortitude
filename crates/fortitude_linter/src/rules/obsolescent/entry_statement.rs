use crate::settings::{CheckSettings, FortranStandard};
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
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
/// ## Notes
/// Entry statements were officially declared obsolescent in Fortran 2008, so
/// this rule only triggers if the target standard is Fortran 2008 or later.
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
    fn check(settings: &CheckSettings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if settings.target_std < FortranStandard::F2008 {
            None
        } else {
            some_vec![Diagnostic::from_node(EntryStatement {}, node)]
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["entry_statement"]
    }
}
