use crate::ast::FortitudeNode;
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for redundant whole-array indexing.
///
/// ## Why is this bad?
/// Adding `(:)` to reference an entire array is redundant. Omitting the
/// subscript makes the same whole-array reference clearer and avoids unnecessary
/// parser/compiler work.
///
/// ## Example
/// ```f90
/// x = x(:)
/// y = y(:, :)
/// ```
///
/// Use instead:
/// ```f90
/// x = x
/// y = y
/// ```
///
/// ## References
/// - [Doctor Fortran: "Doctor, it hurts when I do this!"](https://stevelionel.com/drfortran/2008/03/31/doctor-it-hurts-when-i-do-this/)
#[derive(ViolationMetadata)]
pub(crate) struct WholeArrayIndexing;

impl AlwaysFixableViolation for WholeArrayIndexing {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Avoid redundant whole-array indexing".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove redundant whole-array indexing".to_string()
    }
}

fn has_only_whole_array_extents(node: &Node, src: &str) -> bool {
    let Some(argument_list) = node.child_with_name("argument_list") else {
        return false;
    };

    let mut cursor = argument_list.walk();
    let mut arguments = argument_list.named_children(&mut cursor);
    let Some(first) = arguments.next() else {
        return false;
    };

    first.kind() == "extent_specifier"
        && first.to_text(src).is_some_and(|text| text.trim() == ":")
        && arguments.all(|argument| {
            argument.kind() == "extent_specifier"
                && argument.to_text(src).is_some_and(|text| text.trim() == ":")
        })
}

fn is_assignment_lhs(node: &Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    if parent.kind() != "assignment_statement" {
        return false;
    }

    parent.child_by_field_name("left").is_some_and(|lhs| {
        lhs.start_byte() == node.start_byte() && lhs.end_byte() == node.end_byte()
    })
}

impl AstRule for WholeArrayIndexing {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if is_assignment_lhs(node) || !has_only_whole_array_extents(node, context.source_text()) {
            return None;
        }

        let name = node.named_child(0)?.to_text(context.source_text())?;
        let fix = Fix::safe_edit(node.edit_replacement(context.source_file(), name.to_string()));

        some_vec!(context.create_diagnostic(Self, node).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["call_expression"]
    }
}
