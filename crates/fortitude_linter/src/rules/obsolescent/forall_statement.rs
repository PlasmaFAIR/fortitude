use crate::diagnostics::{Diagnostic, Violation};
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for `forall` statements.
///
/// ## Why is this bad?
/// The F2018 standard made `forall` statements obsolescent in favour of `do
/// concurrent`. They were orginally added with the intention of parallelising
/// loops across multiple processors, however, they turned out to have too many
/// restrictions for compilers to be able to take full advantage of them.
///
/// Instead, the `do concurrent` statement was introduced, which solved many of
/// these difficulties (although not without its own issues, see [1]), along
/// with the use of pointer rank remapping.
///
/// ## Example
/// ```f90
/// forall (i=1:N)
///   b(i) = a(i) * c(i)
/// end forall
/// ```
///
/// Use instead:
/// ```f90
/// do concurrent (i=1:N)
///   b(i) = a(i) * c(i)
/// end do concurrent
/// ```
///
/// ## References
/// - [1]: [`DO CONCURRENT` isn’t necessarily concurrent](https://flang.llvm.org/docs/DoConcurrent.html)
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[derive(ViolationMetadata)]
pub(crate) struct ForallStatement;

impl Violation for ForallStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`forall` statements are obsolescent in F2018, prefer `do concurrent` instead".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `do concurrent` instead".into())
    }
}

impl AstRule for ForallStatement {
    fn check<'a>(context: &'a CheckContext, node: &'a Node) -> Option<Vec<Diagnostic>> {
        // Only made obsolescent in F2018
        if context.settings().target_std < FortranStandard::F2018 {
            return None;
        }

        some_vec![context.create_diagnostic(ForallStatement {}, node.child(0)?)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["forall_statement"]
    }
}
