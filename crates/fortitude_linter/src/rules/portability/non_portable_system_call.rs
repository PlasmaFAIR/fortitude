use crate::diagnostics::{Diagnostic, Fix, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, field};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for use of the non-portable `system` call for running programs.
///
/// ## Why is this bad?
/// `system` is a GFortran extension and isn't available as part of other compilers.
///
/// ## Example
/// ```f90
/// call system("dir")
/// ```
///
/// Use instead:
/// ```f90
/// call execute_command_line("dir")
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableSystemCall;

impl Violation for NonPortableSystemCall {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Use of non-portable `system` call".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Replace with `execute_command_line`".to_string())
    }
}

impl AstRule for NonPortableSystemCall {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let identifier = node.child_by_field_id(field!("subroutine").into()).unwrap();

        // Skip subroutines with other names
        if !(identifier.text().eq_ignore_ascii_case("system")) {
            return None;
        }

        // Exit early if there is a user-defined symbol called 'system'
        if context.symbol_table().get("system").is_some() {
            return None;
        }

        let edit = identifier.edit_replacement("execute_command_line".to_string());

        some_vec!(
            context
                .create_diagnostic(Self {}, identifier)
                .with_fix(Fix::unsafe_edit(edit))
        )
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["subroutine_call"]
    }
}
