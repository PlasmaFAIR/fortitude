use crate::settings::{CheckSettings, FortranStandard};
use crate::{AstRule, FromAstNode, SymbolTables};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
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

        some_vec![Diagnostic::from_node(EquivalenceStatement {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["equivalence_statement"]
    }
}
