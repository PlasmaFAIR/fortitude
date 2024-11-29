use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for statement functions.
///
/// ## Why is this bad?
/// Statement functions are an obsolescent feature from Fortran 77,
/// and have been entirely supplanted by internal
/// procedures. Statement functions are much more limited in what they
/// can do. They were declared obsolescent in Fortran 90 and removed
/// in Fortran 95.
///
/// ## Examples
/// Statement functions are easily replaced with internal procedures:
///
/// ```f90
/// real :: f, x
/// f(x) = x**2 + x
/// ```
/// becomes:
///
/// ```f90
/// contains
///   real function f(x)
///     real, intent(in) :: x
///     f = x**2 + x
///   end function f
/// ```
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[violation]
pub struct StatementFunction {}

impl Violation for StatementFunction {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("statement functions are obsolescent, prefer internal functions")
    }
}

impl AstRule for StatementFunction {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        some_vec![Diagnostic::from_node(StatementFunction {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["statement_function"]
    }
}
