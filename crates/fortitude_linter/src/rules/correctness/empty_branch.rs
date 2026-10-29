use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for `if`, `else if` and `else` branches that contain no statements.
/// Branches that only contain comments are also reported.
///
/// ## Why is this bad?
/// An empty branch is usually left over from code that was removed or commented
/// out, or from something that was forgotten. If the branch is meant to do
/// nothing, say so explicitly with a `continue` statement.
///
/// ## Example
/// ```f90
/// if (flag) then
///     call do_something()
/// else
/// end if
/// ```
///
/// Use instead:
/// ```f90
/// if (flag) then
///     call do_something()
/// end if
/// ```
///
/// Or, if the empty branch is intentional:
/// ```f90
/// if (flag) then
///     call do_something()
/// else
///     continue
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct EmptyBranch {
    branch: String,
}

impl Violation for EmptyBranch {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Empty `{branch}` branch")
    }
}

impl AstRule for EmptyBranch {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if has_statements(node) {
            return None;
        }

        let branch = match node.kind() {
            "if_statement" => "if",
            "elseif_clause" => "else if",
            "else_clause" => "else",
            _ => return None,
        };

        some_vec!(context.create_diagnostic(
            Self {
                branch: branch.to_string(),
            },
            node
        ))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["if_statement", "elseif_clause", "else_clause"]
    }
}

/// Does this branch contain anything besides comments and its own header?
fn has_statements(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            // Reached the end of this branch
            "elseif_clause" | "else_clause" | "end_if_statement" => break,
            // Header parts and comments are not statements
            "comment"
            | "block_label_start_expression"
            | "parenthesized_expression"
            | "identifier" => {}
            _ => return true,
        }
    }
    false
}
