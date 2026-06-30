use crate::ast::{FortitudeNode, types::BlockExit};
use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for `return`, `exit`, `cycle`, and `stop` statements that result in
/// unreachable code.
///
/// ## Why is this bad?
/// Unreachable code can never be executed, and is almost certainly a mistake.
///
/// ## Example
/// ```f90
/// subroutine example(x)
///   integer, intent(inout) :: x
///   x = x + 1
///   return
///   print *, x  ! This statement is unreachable
/// end subroutine example
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UnreachableStatement {
    kind: String,
}

impl Violation for UnreachableStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("code following `{}` is unreachable", self.kind)
    }
}

// TODO: This list may need to be expanded. Doesn't handle preproc or coarray
// statements yet.
pub const EXECUTABLE_STATEMENTS: &[u16] = &[
    kind!("keyword_statement"),
    kind!("stop_statement"),
    kind!("assignment_statement"),
    kind!("pointer_association_statement"),
    kind!("subroutine_call"),
    kind!("read_statement"),
    kind!("write_statement"),
    kind!("print_statement"),
    kind!("open_statement"),
    kind!("close_statement"),
    kind!("format_statement"),
    kind!("inquire_statement"),
    kind!("file_position_statement"),
    kind!("allocate_statement"),
    kind!("deallocate_statement"),
    kind!("if_statement"),
    kind!("do_statement"),
    kind!("select_case_statement"),
    kind!("select_type_statement"),
    kind!("select_rank_statement"),
    kind!("where_statement"),
    kind!("forall_statement"),
    kind!("block_construct"),
    kind!("associate_statement"),
    kind!("nullify_statement"),
];

impl AstRule for UnreachableStatement {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // TODO: Not catching returns at the end of block, associate, etc.
        // This will require going up the tree before finding the next statement.
        let text = node.child(0)?.to_text(context.source_text())?;
        let _ = BlockExit::try_from(text).ok()?;
        let sibling = node.next_non_comment_sibling()?;
        if EXECUTABLE_STATEMENTS.contains(&sibling.kind_id()) {
            some_vec!(context.create_diagnostic(
                UnreachableStatement {
                    kind: text.to_string(),
                },
                sibling,
            ))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["keyword_statement", "stop_statement"]
    }
}
