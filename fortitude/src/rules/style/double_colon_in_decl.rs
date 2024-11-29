use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for missing double-colon separator in variable declarations.
///
/// ## Why is this bad?
/// The double-colon separator is required when declaring variables with
/// attributes, so for consistency, all variable declarations should use it.
#[violation]
pub struct MissingDoubleColon {}

impl AlwaysFixableViolation for MissingDoubleColon {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("variable declaration missing '::'")
    }

    fn fix_title(&self) -> String {
        "Add '::'".to_string()
    }
}
impl AstRule for MissingDoubleColon {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node
            .children(&mut node.walk())
            .filter_map(|child| child.to_text(src.source_text()))
            .all(|child| child != "::")
        {
            let first_decl = node.child_by_field_name("declarator")?;
            let start_pos = TextSize::try_from(first_decl.start_byte()).unwrap();
            let fix = Fix::safe_edit(Edit::insertion(":: ".to_string(), start_pos));
            some_vec!(Diagnostic::from_node(Self {}, node).with_fix(fix))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["variable_declaration"]
    }
}
