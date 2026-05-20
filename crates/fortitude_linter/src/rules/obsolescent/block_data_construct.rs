use crate::diagnostics::{Diagnostic, Violation};
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext};

use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for `block data` constructs.
///
/// ## Why is this bad?
/// Fortran 90 introduced modules, which made `common` blocks redundant, and
/// with them the `block data` construct.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[derive(ViolationMetadata)]
pub(crate) struct BlockDataConstruct;

impl Violation for BlockDataConstruct {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`block data` is obsolescent, use a `module` instead".to_string()
    }
}

impl AstRule for BlockDataConstruct {
    fn check<'a>(context: &'a CheckContext, node: &'a Node) -> Option<Vec<Diagnostic>> {
        // Only made obsolescent in F2018
        if context.settings().target_std < FortranStandard::F2018 {
            return None;
        }

        some_vec![context.create_diagnostic(BlockDataConstruct {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["block_data_statement"]
    }
}
