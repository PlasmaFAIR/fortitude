use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for missing `private` statements in modules
///
/// ## Why is this bad?
/// The `private` statement makes all entities (variables, types, procedures)
/// private by default, requiring an explicit `public` attribute to make them
/// available. As well as improving encapsulation between modules, this also
/// makes it possible to detect unused entities.
#[violation]
pub struct MissingPrivateStatement {
    name: String,
}

impl Violation for MissingPrivateStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("module '{}' missing 'private' statement", self.name)
    }
}

impl AstRule for MissingPrivateStatement {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let module = node.parent()?;

        if module.child_with_name("private_statement").is_none() {
            let name = node.named_child(0)?.to_text(src.source_text())?.to_string();
            return some_vec![Diagnostic::from_node(
                MissingPrivateStatement { name },
                node
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module_statement"]
    }
}
