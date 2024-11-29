use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for common blocks.
///
/// ## Why is this bad?
/// Common blocks are an obsolescent feature from Fortran 77 that may be used
/// to share global data between functions and subroutines. They must be
/// redeclared for each use, and the neither the types nor sizes contained
/// within them are checked between uses. That means that the following code
/// will compile without issue:
///
/// ```f90
/// subroutine s1()
///   common /mydata/ i, j
///   integer(4) :: i, j
///   i = 1
///   j = 0
/// end subroutine s1
///
/// subroutine s2()
///   common /mydata/ x
///   real(8) :: x
///   x = 0.5  ! Overwrite both i and j
/// end subroutine s2
/// ```
///
/// Code using common blocks can therefore be unwieldy and error-prone. The
/// use of modules obviates thier use.
///
/// Derived types may also be used to encapsulate a set of related data, and
/// this approach also helps to improve encapsulation.
///
/// ## Examples
///
/// ```f90
/// subroutine s()
///   common /mydata/ i, j
///   integer :: i, j
///   i = 1
/// end subroutine s
/// ```
/// becomes:
///
/// ```f90
/// module mydata
///   implicit none
///   public
///   integer :: i, j
/// end module mydata
///
/// subroutine s()
///   use mydata, only: i, j
///   i = 1
/// end subroutine s
/// ```
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[violation]
pub struct CommonBlock {}

impl Violation for CommonBlock {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("common blocks are obsolescent, prefer modules or derived types")
    }
}

impl AstRule for CommonBlock {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        some_vec![Diagnostic::from_node(CommonBlock {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["common_statement"]
    }
}
