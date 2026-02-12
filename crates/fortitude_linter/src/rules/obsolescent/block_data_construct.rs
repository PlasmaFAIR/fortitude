use crate::settings::{CheckSettings, FortranStandard};
use crate::{AstRule, FromAstNode, SymbolTables};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
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
    fn check<'a>(
        settings: &CheckSettings,
        node: &'a Node,
        _src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // Only made obsolescent in F2018
        if settings.target_std < FortranStandard::F2018 {
            return None;
        }

        some_vec![Diagnostic::from_node(BlockDataConstruct {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["block_data_statement"]
    }
}
