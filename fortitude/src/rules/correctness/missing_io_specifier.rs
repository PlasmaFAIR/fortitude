use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
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
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let txt = src.source_text();
        for arg in node.named_children(&mut node.walk()) {
            if arg.kind() == "keyword_argument" {
                if let Some(key) = arg.child_by_field_name("name") {
                    if let Some("action") = key.to_text(txt) {
                        return None;
                    }
                }
            }
        }
        some_vec![Diagnostic::from_node(Self {}, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["open_statement"]
    }
}
