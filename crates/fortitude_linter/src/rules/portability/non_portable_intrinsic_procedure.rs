use crate::diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
use crate::settings::FortranStandard;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use fortitude_sitter::Node;
use fortitude_sitter::ast::types::{HasName, ProcedureCall};
use fortitude_sitter::traits::{HasNode, TextRanged};
use ruff_diagnostics::Edit;
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
    // TODO(peter): make non-option
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
    procedure: ProcedureCall,
    replacement: &str,
) -> Option<Vec<Diagnostic>> {
    let node = procedure.name().node();
    let edit = node.edit_replacement(replacement.to_string());
    let replacement = Some(replacement.to_string());
    let name = procedure.name().to_string();

    some_vec![
        context
            .create_diagnostic(NonPortableIntrinsicProcedure { name, replacement }, node)
            .with_fix(Fix::unsafe_edit(edit)),
    ]
}

/// Create diagnostic with a suggestion but no auto-fix
fn create_non_replacement(
    context: &CheckContext,
    procedure: ProcedureCall,
    replacement: &str,
) -> Option<Vec<Diagnostic>> {
    let help = format!(
        "The standard `{replacement}` has different semantics. See the docs for more details"
    );
    let replacement = Some(replacement.to_string());
    let name = procedure.name().to_string();

    let mut diagnostic = context.create_diagnostic(
        NonPortableIntrinsicProcedure { name, replacement },
        procedure.name().node(),
    );
    diagnostic.help(help);

    some_vec![diagnostic]
}

/// Create diagnostic with a straight-forward replacement
fn system_replacement(context: &CheckContext, procedure: ProcedureCall) -> Option<Vec<Diagnostic>> {
    let node = procedure.name().node();
    let replacement = "execute_command_line";
    let edit = node.edit_replacement(replacement.to_string());
    let replacement = Some(replacement.to_string());
    let name = procedure.name().to_string();

    let mut rest = vec![];

    if let Some(arg) = procedure.arg(1) {
        // if we have two positional arguments, the second one is status and needs
        // keyword `exitstat=` appending

        let arg_edit = Edit::insertion("exitstat=".to_string(), arg.start_textsize());
        rest.push(arg_edit);
    } else if let Some(status) = procedure.kwarg("status") {
        // if we have kwarg status, it needs replacing with `exitstat`
        let kwarg_edit = status
            .name()
            .node()
            .edit_replacement("exitstat".to_string());
        rest.push(kwarg_edit);
    }

    let fix = Fix::unsafe_edits(edit, rest);

    some_vec![
        context
            .create_diagnostic(NonPortableIntrinsicProcedure { name, replacement }, node)
            .with_fix(fix)
    ]
}

/// Work out the `stop` or `error stop` statement that replaces a call to `exit` or
/// `abort`, along with the minimum standard that supports the replacement
fn stop_replacement(procedure: &ProcedureCall, keyword: &str) -> (Option<String>, FortranStandard) {
    if procedure.args().is_empty() && procedure.kwargs().is_empty() {
        return (Some(keyword.to_string()), FortranStandard::default());
    };

    // More than one argument: no fix
    if procedure.args().len() + procedure.kwargs().len() > 1 {
        return (None, FortranStandard::default());
    }

    let arg = if let Some(arg) = procedure.arg(0) {
        arg
    } else if let Some(kwarg) = procedure.kwarg("status") {
        kwarg.value()
    } else {
        return (None, FortranStandard::default());
    };

    let text = arg.text();
    match arg.kind_id() {
        kind!("number_literal") => {
            // Only plain integers: no kind suffixes, no real numbers
            let is_integer = text.bytes().all(|b| b.is_ascii_digit());
            // Before Fortran 2008, a stop code has at most 5 digits
            let min_standard = if text.len() <= 5 {
                FortranStandard::default()
            } else {
                FortranStandard::F2008
            };

            (
                is_integer.then(|| format!("{keyword} {text}")),
                min_standard,
            )
        }
        // String literals are valid stop codes in every standard
        kind!("string_literal") => (
            Some(format!("{keyword} {text}")),
            FortranStandard::default(),
        ),
        // Any other expression is only allowed from Fortran 2018
        _ => (Some(format!("{keyword} {text}")), FortranStandard::F2018),
    }
}

/// Create diagnostic for non-portable `stop`/`error stop` type calls, taking
/// into account the target standard
fn create_stop_replacement(
    context: &CheckContext,
    procedure: ProcedureCall,
) -> Option<Vec<Diagnostic>> {
    let name = procedure.name().as_str();
    let node = procedure.node();
    let is_abort = name.eq_ignore_ascii_case("abort");
    let standard = if is_abort { "error stop" } else { "stop" };

    let (replacement, min_standard) = stop_replacement(&procedure, standard);

    let mut diagnostic = context.create_diagnostic(
        NonPortableIntrinsicProcedure {
            name: name.to_string(),
            replacement: replacement.clone(),
        },
        node,
    );

    if let Some(text) = replacement {
        let min_std = if is_abort {
            // `error stop` was added in Fortran 2008
            FortranStandard::F2008.max(min_standard)
        } else {
            min_standard
        };

        let edit = node.edit_replacement(text);
        let target_std = context.settings().target_std;
        let fix = if target_std < min_std {
            diagnostic.info(format!("This fix requires Fortran standard {min_std} which is above the target standard of {target_std}"));
            Fix::display_only_edit(edit)
        } else {
            Fix::unsafe_edit(edit)
        };

        diagnostic = diagnostic.with_fix(fix);
    } else {
        diagnostic.help(format!("Replace with `{standard}`"));
    }
    some_vec!(diagnostic)
}

impl AstRule for NonPortableIntrinsicProcedure {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let Ok(procedure) = ProcedureCall::try_from_node(node) else {
            return None;
        };

        // Exit early if there is a user-defined symbol with this name
        if context.symbol_table().get(procedure.name()).is_some() {
            return None;
        }
        let is_subroutine = procedure.kind().is_subroutine();

        // TODO: add min std to all replacements

        match procedure.name().as_lower().as_ref() {
            "system" if is_subroutine => system_replacement(context, procedure),
            // `system` as a function -- could be replaced by subroutine if not in expression
            "system" => create_non_replacement(context, procedure, "execute_command_line"),
            "abort" | "exit" if is_subroutine => create_stop_replacement(context, procedure),
            "srand" if is_subroutine => create_non_replacement(context, procedure, "random_seed"),
            "rand" if !is_subroutine => create_non_replacement(context, procedure, "random_number"),
            "iargc" if !is_subroutine => create_replacement(context, procedure, "command_argument_count"),
            _ => None,
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression", "subroutine_call"]
    }
}
