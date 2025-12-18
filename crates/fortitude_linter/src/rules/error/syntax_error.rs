use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for syntax errors
///
/// This rule reports any syntax errors reported by Fortitude's Fortran parser.
/// This may indicate an error with your code, an aspect of Fortran not recognised
/// by the parser, or a non-standard extension to Fortran that our parser can't
/// handle, such as a pre-processor.
///
/// If this rule is reporting valid Fortran, please let us know, as it's likely a
/// bug in our code or in our parser!
#[derive(ViolationMetadata)]
pub(crate) struct SyntaxError {}

impl Violation for SyntaxError {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Syntax error".to_string()
    }
}

impl AstRule for SyntaxError {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        _src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        some_vec![Diagnostic::from_node(Self {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["ERROR"]
    }
}
