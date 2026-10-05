use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What does it do?
/// Checks for `print` statements and suggests using `write` instead.
///
/// ## Why is this bad?
/// `write` is more general and consistent with Fortran file I/O.
#[derive(ViolationMetadata)]
pub(crate) struct PrintStatement;
impl AlwaysFixableViolation for PrintStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Prefer `write` over `print`".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace `print` with `write`".to_string()
    }
}
impl AstRule for PrintStatement {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let format_node = node.child_with_name("format_identifier")?;
        let format_text = format_node.text();

        let output_text = node
            .child_with_name("output_item_list")
            .map(|items| items.text());

        let replacement = match output_text {
            Some(items) => format!("write (*, {format_text}) {items}"),
            None => format!("write (*, {format_text})"),
        };

        let fix = Fix::safe_edit(node.edit_replacement(replacement));

        some_vec![
            context
                .create_diagnostic(PrintStatement, node)
                .with_fix(fix)
        ]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["print_statement"]
    }
}
