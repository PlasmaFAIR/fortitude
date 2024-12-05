use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
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

impl AlwaysFixableViolation for OldStyleArrayLiteral {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Array literal uses old-style syntax: prefer `[...]`")
    }

    fn fix_title(&self) -> String {
        "Change `(/.../)` to `[...]`".to_string()
    }
}

impl AstRule for OldStyleArrayLiteral {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let open_bracket = node.child(0)?;

        if open_bracket.to_text(src.source_text())?.starts_with("(/") {
            let close_bracket = node.children(&mut node.walk()).last()?;

            let edit_open = open_bracket.edit_replacement("[".to_string());
            let edit_close = close_bracket.edit_replacement("]".to_string());
            let fix = Fix::safe_edits(edit_open, [edit_close]);

            return some_vec!(Diagnostic::from_node(Self {}, node).with_fix(fix));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["array_literal"]
    }
}
