use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use std::path::Path;
use tree_sitter::Node;

/// ## What it does
/// Checks for the use of the deprecated `omp_lib.h` include.
///
/// ## Why is this bad?
/// OpenMP deprecated the `omp_lib.h` include file in OpenMP 6.0. It is recommended
/// to switch to the `omp_lib` module
///
/// The `omp_lib` module should be a drop-in replacement for the `omp_lib.h` include (except
/// for its placement in the Fortran file).
///
/// ## References
/// OpenMP Architecture Review Board, OpenMP Application Programming Interface 6.0,
/// Nov. 2024. Available: https://www.openmp.org/wp-content/uploads/OpenMP-API-Specification-6-0.pdf

#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedOmpInclude {}

impl Violation for DeprecatedOmpInclude {
    #[derive_message_formats]
    fn message(&self) -> String {
        "omp_lib.h include is deprecated, use the omp_lib module instead".to_string()
    }
}

impl AstRule for DeprecatedOmpInclude {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        _src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let include_file = node
            .child_with_name("filename")?
            .to_text(_src.source_text())?
            .to_lowercase();

        // Strip quotes from the include file name
        let include_file = include_file.trim_matches('"').trim_matches('\'');

        if Path::new(&include_file).file_name() == Some("omp_lib.h".as_ref()) {
            return some_vec![Diagnostic::from_node(DeprecatedOmpInclude {}, node)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["include_statement"]
    }
}
