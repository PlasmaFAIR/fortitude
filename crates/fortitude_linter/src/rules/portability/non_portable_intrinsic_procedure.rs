use crate::diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
use crate::rules::utilities::is_assignment_lhs;
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, field, kind};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for use of non-portable intrinsic procedures.
///
/// ## Why is this bad?
/// Many compilers include extra intrinsic procedures which provide
/// functionality which was not in the Fortran standard at the time of the
/// compiler's release, although they often become standardised in later
/// versions. These extensions may not be available in other compilers, or may
/// work differently, making the code non-portable.
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
///
/// ## Extensions and their standard equivalents
///
/// | Extension          | Standard                       | Notes |
/// |--------------------|--------------------------------|-------|
/// | [`system`][system] | [`execute_command_line`][execute_command_line] | Extension can be either subroutine or function, standard version is a subroutine only |
/// | [`exit`][exit]     | `stop`                           | Extension is a subroutine, standard version is a statement |
/// | [`abort`][abort]   | `error stop`                     | Extension is a subroutine, standard version is a statement |
/// | [`srand`][srand]   | [`random_seed`][random_seed]     | Standard has different semantics |
/// | [`rand`][rand]     | [`random_number`][random_number] | Standard is a subroutine rather than function |
///
/// ## Fix safety
/// ### `system`
/// When the gfortran `system` extension is used as a subroutine, it has nearly
/// identical semantics to the standard `execute_command_line`, except that the
/// `status` argument should be replaced with the `exitstat` keyword argument
/// instead.
///
/// However, `system` can also be used as a function, in which case a fix may be
/// more complicated if used as part of a larger expression.
///
/// ### `exit` and `abort`
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
/// ### `srand` and `rand`
/// ```f90
/// call srand(seed)
/// x = rand(seed)
/// ```
///
/// #### Use instead
/// ```f90
/// integer, allocatable :: seed(:)
/// integer :: n
///
/// call random_seed(size = n)
/// allocate(seed(n))
/// seed(1) = 12345
/// call random_seed(put=seed)
/// call random_number(x)
/// ```
///
/// Prefer using the Fortran standard intrinsics `random_seed` and `random_number` instead.
/// These are not interchangeable replacements, as for example, `random_number(x)` is a subroutine
/// that modifies `x`, whereas `rand()` is a function that returns a single `real`.
///
/// ## References
/// - [GNU Fortran intrinsic procedures](https://gcc.gnu.org/onlinedocs/gfortran/Intrinsic-Procedures.html)
///
/// [abort]: https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html
/// [execute_command_line]: https://gcc.gnu.org/onlinedocs/gfortran/EXECUTE_005fCOMMAND_005fLINE.html
/// [exit]: https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html
/// [rand]: https://gcc.gnu.org/onlinedocs/gfortran/RAND.html
/// [random_number]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html
/// [random_seed]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html
/// [srand]: https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html
/// [system]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM.html
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableIntrinsicProcedure {
    /// Extension name
    name: String,
    /// Standard name
    replacement: Option<String>,
}

impl Violation for NonPortableIntrinsicProcedure {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, .. } = self;
        format!("Use of non-portable `{name}` call")
    }

    fn fix_title(&self) -> Option<String> {
        self.replacement
            .as_ref()
            .map(|replacement| format!("Replace with `{replacement}`"))
    }
}

/// Create diagnostic with a straight-forward replacement
fn create_replacement(
    context: &CheckContext,
    identifier: &Node,
    name: String,
    replacement: &str,
) -> Option<Vec<Diagnostic>> {
    let edit = identifier.edit_replacement(replacement.to_string());
    let replacement = Some(replacement.to_string());

    some_vec![
        context
            .create_diagnostic(
                NonPortableIntrinsicProcedure { name, replacement },
                identifier
            )
            .with_fix(Fix::unsafe_edit(edit)),
    ]
}

/// Create diagnostic with a suggestion but no auto-fix
fn create_non_replacement(
    context: &CheckContext,
    identifier: &Node,
    name: String,
    replacement: &str,
) -> Option<Vec<Diagnostic>> {
    let help = format!(
        "The standard `{replacement}` has different semantics. See the docs for more details"
    );
    let replacement = Some(replacement.to_string());

    let mut diagnostic = context.create_diagnostic(
        NonPortableIntrinsicProcedure { name, replacement },
        identifier,
    );
    diagnostic.help(help);

    some_vec![diagnostic]
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

/// Create diagnostic for non-portable `stop`/`error stop` type calls, taking
/// into account the target standard
fn create_stop_replacement(
    context: &CheckContext,
    node: &Node,
    name: String,
) -> Option<Vec<Diagnostic>> {
    let replacement = if name == "abort" {
        // `error stop` was added in Fortran 2008
        if context.settings().target_std >= FortranStandard::F2008 {
            stop_replacement(context, node, "error stop")
        } else {
            None
        }
    } else {
        stop_replacement(context, node, "stop")
    };

    let mut diagnostic = context.create_diagnostic(
        NonPortableIntrinsicProcedure {
            name: name.clone(),
            replacement: replacement.clone(),
        },
        node,
    );
    if let Some(text) = replacement {
        diagnostic = diagnostic.with_fix(Fix::unsafe_edit(node.edit_replacement(text)));
    } else {
        let replacement = if name == "abort" {
            "error stop"
        } else {
            "stop"
        };
        diagnostic.help(format!("Replace with `{replacement}`"));
    }
    some_vec!(diagnostic)
}

fn is_array_access(node: &Node) -> bool {
    let child_node = node.child_with_name("argument_list");
    match child_node {
        Some(node) => node.child_with_name("extent_specifier").is_some(),
        None => false,
    }
}

impl AstRule for NonPortableIntrinsicProcedure {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let is_subroutine = node.kind_id() == kind!("subroutine_call");

        let identifier = if is_subroutine {
            node.child_by_field_id(field!("subroutine").into()).unwrap()
        } else {
            // Check that this is not actually an array access/assignment
            if is_assignment_lhs(node) || is_array_access(node) {
                return None;
            }

            node.child_with_name("identifier").unwrap()
        };

        let name = identifier.text().to_ascii_lowercase();

        // Exit early if there is a user-defined symbol with this name
        if context.symbol_table().get(&name).is_some() {
            return None;
        }

        match name.as_str() {
            "system" if is_subroutine => {
                // TODO(peter): also handle `status` argument
                create_replacement(context, &identifier, name, "execute_command_line")
            }
            // `system` as a function -- could be replaced by subroutine if not in expression
            "system" => create_non_replacement(context, &identifier, name, "execute_command_line"),
            "abort" | "exit" if is_subroutine => create_stop_replacement(context, node, name),
            "srand" if is_subroutine => {
                create_non_replacement(context, &identifier, name, "random_seed")
            }
            "rand" if !is_subroutine => {
                create_non_replacement(context, &identifier, name, "random_number")
            }
            _ => None,
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression", "subroutine_call"]
    }
}
