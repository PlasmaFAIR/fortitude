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
/// A fix is only offered when the target standard supports it and the call has
/// at most one positional argument. A string literal is always accepted as a
/// stop code. An integer literal is too, except that literals with more than 5
/// digits need Fortran 2008. Any other expression requires Fortran 2018, and
/// `error stop` requires Fortran 2008. Some compilers (such as Intel and NAG)
/// accept an argument to `abort`, whereas gfortran does not.
///
/// ## References
/// - [GFortran docs for `exit`](https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html)
/// - [GFortran docs for `abort`](https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html)
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

/// Work out the `stop` or `error stop` statement that replaces a call to `exit` or
/// `abort`, if there is a safe way to write one for the targeted standard.
fn stop_replacement(context: &CheckContext, node: &Node, keyword: &str) -> Option<String> {
    let Some(arguments) = node.child_with_id(kind!("argument_list")) else {
        return Some(keyword.to_string());
    };

    let mut cursor = arguments.walk();
    let mut args = arguments
        .named_children(&mut cursor)
        .filter(|arg| arg.kind_id() != kind!("comment"));

    let Some(arg) = args.next() else {
        return Some(keyword.to_string());
    };

    // More than one argument: no fix
    if args.next().is_some() {
        return None;
    }

    let text = arg.text();
    match arg.kind_id() {
        kind!("keyword_argument") => None,
        kind!("number_literal") => {
            // Only plain integers: no kind suffixes, no real numbers
            let is_integer = text.bytes().all(|b| b.is_ascii_digit());
            // Before Fortran 2008, a stop code has at most 5 digits
            let fits = text.len() <= 5 || context.settings().target_std >= FortranStandard::F2008;
            (is_integer && fits).then(|| format!("{keyword} {text}"))
        }
        // String literals are valid stop codes in every standard
        kind!("string_literal") => Some(format!("{keyword} {text}")),
        // Any other expression is only allowed from Fortran 2018
        _ if context.settings().target_std >= FortranStandard::F2018 => {
            Some(format!("{keyword} {text}"))
        }
        _ => None,
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
            if context.settings().target_std >= FortranStandard::F2008 {
                stop_replacement(context, node, "error stop")
            } else {
                None
            }
        } else {
            stop_replacement(context, node, "stop")
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
