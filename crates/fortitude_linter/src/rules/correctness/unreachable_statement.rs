use crate::ast::{FortitudeNode, types::BlockExit};
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
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
pub const EXECUTABLE_STATEMENTS: &[&str] = &[
    "keyword_statement",
    "stop_statement",
    "assignment_statement",
    "pointer_association_statement",
    "subroutine_call",
    "read_statement",
    "write_statement",
    "print_statement",
    "open_statement",
    "close_statement",
    "format_statement",
    "inquire_statement",
    "file_position_statement",
    "allocate_statement",
    "deallocate_statement",
    "if_statement",
    "do_statement",
    "select_case_statement",
    "select_type_statement",
    "select_rank_statement",
    "where_statement",
    "forall_statement",
    "block_construct",
    "associate_statement",
    "nullify_statement",
];

impl AstRule for UnreachableStatement {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_tables: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // TODO: Not catching returns at the end of block, associate, etc.
        // This will require going up the tree before finding the next statement.
        let text = node.child(0)?.to_text(src.source_text())?;
        let _ = BlockExit::try_from(text).ok()?;
        let sibling = node.next_non_comment_sibling()?;
        if EXECUTABLE_STATEMENTS.contains(&sibling.kind()) {
            some_vec!(Diagnostic::from_node(
                UnreachableStatement {
                    kind: text.to_string(),
                },
                &sibling,
            ))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["keyword_statement", "stop_statement"]
    }
}
