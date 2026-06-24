use crate::ast::FortitudeNode;
use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.kwarg_exists("action", context.source_text()) {
            return None;
        }
        some_vec![context.create_diagnostic(Self {}, node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["open_statement"]
    }
}
