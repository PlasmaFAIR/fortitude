use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for missing double-colon separator in variable declarations.
///
/// ## Why is this bad?
/// The double-colon separator is required when declaring variables with
/// attributes, so for consistency, all variable declarations should use it.
#[violation]
pub struct MissingDoubleColon {}

impl Violation for MissingDoubleColon {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("variable declaration missing '::'")
    }
}
impl AstRule for MissingDoubleColon {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node
            .children(&mut node.walk())
            .filter_map(|child| child.to_text(src.source_text()))
            .all(|child| child != "::")
        {
            some_vec!(Diagnostic::from_node(Self {}, node))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["variable_declaration"]
    }
}
