use crate::ast::FortitudeNode;
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for missing double-colon separator in variable declarations.
///
/// ## Why is this bad?
/// The double-colon separator is required when declaring variables with
/// attributes, so for consistency, all variable declarations should use it.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDoubleColon {}

impl AlwaysFixableViolation for MissingDoubleColon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "variable declaration missing '::'".to_string()
    }

    fn fix_title(&self) -> String {
        "Add '::'".to_string()
    }
}
impl AstRule for MissingDoubleColon {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node
            .children(&mut node.walk())
            .filter_map(|child| child.to_text(context.source_text()))
            .all(|child| child != "::")
        {
            let first_decl = node.child_by_field_name("declarator")?;
            let start_pos = first_decl.start_textsize();
            let fix = Fix::safe_edit(Edit::insertion(":: ".to_string(), start_pos));
            some_vec!(context.create_diagnostic(Self {}, node).with_fix(fix))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["variable_declaration"]
    }
}
