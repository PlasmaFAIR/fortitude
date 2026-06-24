use crate::diagnostics::{Diagnostic, Violation};
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of `equivalence` statements.
///
/// ## Why is this bad?
/// Prior to Fortran 90, `equivalence` was a versatile and powerful statement,
/// but error-prone and easily abused. Fortran 90 introduced many safer features
/// which have made `equivalence` redundant, and Fortran 2018 officially made
/// the statement obsolescent.
///
/// Depending on its use case, `equivalence` statements should be replaced with:
/// - automatic arrays,
/// - allocatable arrays,
/// - pointers to reuse storage,
/// - pointers as aliases,
/// - or the `transfer` function for bit manipulation.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[derive(ViolationMetadata)]
pub(crate) struct EquivalenceStatement;

impl Violation for EquivalenceStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`equivalence` is obsolete and should not be used".to_string()
    }
}

impl AstRule for EquivalenceStatement {
    fn check<'a>(context: &'a CheckContext, node: &'a Node) -> Option<Vec<Diagnostic>> {
        // Only made obsolescent in F2018
        if context.settings().target_std < FortranStandard::F2018 {
            return None;
        }

        some_vec![context.create_diagnostic(EquivalenceStatement {}, node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["equivalence_statement"]
    }
}
