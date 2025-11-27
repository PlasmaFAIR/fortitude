use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for unnecessary `return` statements
///
/// ## Why is this bad?
/// Unlike many other languages, Fortran's `return` statement is only used to
/// return early from procedures, and not to return values. If a `return`
/// statement is the last executable statement in a procedure, it can safely be
/// removed.
///
/// ## Example
/// ```f90
/// integer function capped_add(a, b)
///   integer, intent(in) :: a, b
///   if ((a + b) > 10) then
///     capped_add = 10
///     return
///   end if
///   capped_add = a + b
///   return   ! This `return` statement does nothing
/// end function capped_add
/// ```
///
/// Use instead:
/// ```f90
/// integer function capped_add(a, b)
///   integer, intent(in) :: a, b
///   if ((a + b) > 10) then
///     capped_add = 10
///     return
///   end if
///   capped_add = a + b
/// end function capped_add
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UselessReturn;

impl AlwaysFixableViolation for UselessReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        "useless `return` statement`".to_string()
    }

    fn fix_title(&self) -> String {
        "remove `return` statement".to_string()
    }
}

impl AstRule for UselessReturn {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        if !node
            .to_text(src.source_text())?
            .eq_ignore_ascii_case("return")
        {
            return None;
        }
        let mut sibling = node.next_named_sibling();
        while let Some(next_sibling) = sibling {
            if next_sibling.kind() != "comment" {
                break;
            }
            sibling = next_sibling.next_named_sibling();
        }

        if !matches!(
            sibling?.kind(),
            "end_function_statement" | "end_subroutine_statement"
        ) {
            return None;
        }
        let edit = node.edit_delete(src);
        some_vec!(Diagnostic::from_node(Self, node).with_fix(Fix::safe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["keyword_statement"]
    }
}
