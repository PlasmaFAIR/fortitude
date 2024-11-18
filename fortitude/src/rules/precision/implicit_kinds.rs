use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks that `real` variables have their kind explicitly specified
///
/// ## Why is this bad?
/// Real variable declarations without an explicit kind will have a compiler/platform
/// dependent precision, which hurts portability and may lead to surprising loss of
/// precision in some cases.
#[violation]
pub struct ImplicitRealKind {
    dtype: String,
}

impl Violation for ImplicitRealKind {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ImplicitRealKind { dtype } = self;
        format!("{dtype} has implicit kind")
    }
}

impl AstRule for ImplicitRealKind {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let dtype = node.child(0)?.to_text(src.source_text())?.to_lowercase();

        if !matches!(dtype.as_str(), "real" | "complex") {
            return None;
        }

        if node.child_by_field_name("kind").is_some() {
            return None;
        }

        some_vec![Diagnostic::from_node(ImplicitRealKind { dtype }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
