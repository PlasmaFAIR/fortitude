use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for missing action specifier when opening files.
///
/// ## Why is this bad?
/// By default, files are opened in `readwrite` mode, but this may not be the
/// programmer's intent. Explicitly specifying `read`, `write` or `readwrite`
/// makes it clear how the file is intended to be used, and prevents the
/// accidental overwriting of input data.
#[derive(ViolationMetadata)]
pub(crate) struct MissingActionSpecifier {}

impl Violation for MissingActionSpecifier {
    #[derive_message_formats]
    fn message(&self) -> String {
        "file opened without action specifier".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'action=read', 'action=write', or 'action=readwrite'".to_string())
    }
}

impl AstRule for MissingActionSpecifier {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        if node.kwarg_exists("action", src.source_text()) {
            return None;
        }
        some_vec![Diagnostic::from_node(Self {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["open_statement"]
    }
}
