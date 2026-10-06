use crate::diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, field, kind};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for use of the non-portable `exit` and `abort` subroutines.
///
/// ## Why is this bad?
/// `exit` and `abort` are GNU extensions and aren't available in other
/// compilers. The standard `stop` and `error stop` statements should be used
/// instead.
///
/// ## Example
/// ```f90
/// call exit(1)
/// call abort
/// ```
///
/// Use instead:
/// ```f90
/// stop 1
/// error stop
/// ```
///
/// ## Fix safety
/// The fix is unsafe because the replacements are not exact equivalents:
/// `stop` prints a `STOP` message to stderr, whereas `call exit` is silent, and
/// `error stop` terminates normally with a non-zero exit code, whereas `call
/// abort` raises a signal and may produce a core dump. The fix also can't tell
/// whether `exit` or `abort` refers to an external procedure of the same name.
///
/// The fix is only offered when the target standard supports it: `stop` with no
/// code or an integer literal is always offered, `stop` with any other
/// expression requires Fortran 2018, and `error stop` requires Fortran 2008.
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableExitCall {
    routine: String,
}

impl Violation for NonPortableExitCall {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Use of non-portable `{}` subroutine", self.routine)
    }

    fn fix_title(&self) -> Option<String> {
        let replacement = if self.routine == "abort" {
            "error stop"
        } else {
            "stop"
        };
        Some(format!("Replace with `{replacement}`"))
    }
}

/// Work out the `stop` statement that replaces `call exit(...)`, if there is
/// a safe way to write one for the targeted standard.
fn exit_replacement(context: &CheckContext, node: &Node) -> Option<String> {
    let Some(arguments) = node.child_with_id(kind!("argument_list")) else {
        return Some("stop".to_string());
    };
    let text = arguments.text();
    let inner = text.trim().strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some("stop".to_string());
    }
    // Keyword or multiple arguments: leave these alone
    if inner.contains('=') || inner.contains(',') {
        return None;
    }
    // A stop code must be a constant expression before Fortran 2018. We can
    // only be sure about plain integer literals.
    let is_literal = inner.bytes().all(|b| b.is_ascii_digit());
    if is_literal || context.settings().target_std >= FortranStandard::F2018 {
        Some(format!("stop {inner}"))
    } else {
        None
    }
}

impl AstRule for NonPortableExitCall {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let identifier = node.child_by_field_id(field!("subroutine").into())?;
        let routine = identifier.text().to_ascii_lowercase();

        // Skip subroutines with other names
        if routine != "exit" && routine != "abort" {
            return None;
        }

        // Exit early if there is a user-defined symbol with the same name
        if context.symbol_table().get(&routine).is_some() {
            return None;
        }

        let replacement = if routine == "abort" {
            // `error stop` was added in Fortran 2008
            (context.settings().target_std >= FortranStandard::F2008)
                .then(|| "error stop".to_string())
        } else {
            exit_replacement(context, node)
        };

        let mut diagnostic = context.create_diagnostic(Self { routine }, node);
        if let Some(text) = replacement {
            diagnostic = diagnostic.with_fix(Fix::unsafe_edit(node.edit_replacement(text)));
        }
        some_vec!(diagnostic)
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["subroutine_call"]
    }
}
