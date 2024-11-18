use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for old style array literals
///
/// ## Why is this bad?
/// Fortran 2003 introduced a shorter syntax for array literals: `[...]`. While the
/// older style, `(/.../)`, is still valid, the F2003 style is shorter and easier to
/// match.
#[violation]
pub struct OldStyleArrayLiteral {}

impl Violation for OldStyleArrayLiteral {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Array literal uses old-style syntax: prefer `[...]`")
    }
}
impl AstRule for OldStyleArrayLiteral {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.to_text(src.source_text())?.starts_with("(/") {
            return some_vec!(Diagnostic::from_node(Self {}, node));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["array_literal"]
    }
}
