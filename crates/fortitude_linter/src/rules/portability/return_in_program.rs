use crate::ast::FortitudeNode;
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of `return` statement inside a `program` body, as allowed by some compilers.
/// Suggests to replace it with `stop`.
///
/// ## Why is this bad?
/// It is non-standard and not portable.
///
/// ## Example
/// ```f90
/// program test
///   return
/// end program test
/// ```
///
/// Use instead:
/// ```f90
/// program test
///   stop
/// end program test
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct ReturnInProgram;

impl AlwaysFixableViolation for ReturnInProgram {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'return' statement in program body".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace 'return' with 'stop'".to_string()
    }
}

impl AstRule for ReturnInProgram {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if !node
            .child(0)?
            .to_text(context.source_text())?
            .eq_ignore_ascii_case("return")
        {
            return None;
        }

        for ancestor in node.ancestors() {
            if matches!(
                ancestor.kind(),
                "function" | "subroutine" | "module" | "submodule" | "block_data"
            ) {
                return None;
            }
        }

        let fix = Fix::safe_edit(node.edit_replacement(context.source_file(), "stop".to_string()));
        some_vec!(context.create_diagnostic(Self {}, node).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["keyword_statement"]
    }
}
