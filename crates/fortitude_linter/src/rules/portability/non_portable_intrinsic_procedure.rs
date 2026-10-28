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
/// It is inherently tricky to identify compiler extensions, different compilers
/// may have intrinsic procedures with the same name but different behaviour for
/// example. So although we have automatic fixes for many of these, we recommend
/// that they are always reviewed by a human; hence we mark all the fixes as
/// [unsafe](../linter.md#fix-safety).
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
/// Summary of compiler-specific intrinsics and their standard equivalents:
///
/// | Extension            | Standard                                           |
/// |----------------------|----------------------------------------------------|
/// | [`system`][system]   | [`execute_command_line`][execute_command_line]     |
/// | [`exit`][exit]       | `stop`                                             |
/// | [`abort`][abort]     | `error stop`                                       |
/// | [`srand`][srand]     | [`random_seed`][random_seed]                       |
/// | [`rand`][rand]       | [`random_number`][random_number]                   |
/// | [`iargc`][iargc]     | [`command_argument_count`][command_argument_count] |
/// | [`getarg`][getarg]   | [`get_command_argument`][get_command_argument]     |
/// | [`etime`][etime]     | Various, see below                                 |
/// | Time/date procedures | [`date_and_time`][date_and_time]                   |
/// | [`lnblnk`][lnblnk]   | [`len_trim`][len_trim]                             |
///
/// See below for details on the specific procedures:
///
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
/// ### `iargc` and `getarg`
/// These are completely replaced with the standard `command_argument_count` and
/// `get_command_argument`, and are almost always a straight replacement,
/// although some compilers may support additional arguments to `getarg`.
///
/// ### `etime`
/// Different compilers may support different semantics `etime`, including
/// calling as either a subroutine or function, and different meanings of the
/// return arguments. There is no straight replacement for `etime` in the
/// standard, but the following intrinsic procedures have similar functionality:
///
/// - [`cpu_time`][cpu_time]: CPU time elapsed in seconds
/// - [`date_and_time`][date_and_time]: Date and time in a standard format
/// - [`system_clock`][system_clock]: Possibly monotonic nanosecond-resolution clock, if available
///
/// MPI and OpenMP also have standard facilities for getting elapsed time, which
/// may be higher resolution.
///
/// ### Time and date procedures
/// There are many different procedures for getting or formatting the time and
/// date in various ways, such as `itime`, `fdate`, `ctime`, and so on. They are
/// mostly all replaced with the standard [`date_and_time`][date_and_time].
///
/// ## References
/// - [GNU Fortran intrinsic procedures](https://gcc.gnu.org/onlinedocs/gfortran/Intrinsic-Procedures.html)
/// - [Intel Fortran language reference](https://www.intel.com/content/www/us/en/docs/fortran-compiler/developer-guide-reference/2026-1/a-to-z-reference.html)
///
/// [abort]: https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html
/// [command_argument_count]: https://gcc.gnu.org/onlinedocs/gfortran/COMMAND_005fARGUMENT_005fCOUNT.html
/// [cpu_time]: https://gcc.gnu.org/onlinedocs/gfortran/CPU_005fTIME.html
/// [date_and_time]: https://gcc.gnu.org/onlinedocs/gfortran/DATE_005fAND_005fTIME.html
/// [etime]: https://gcc.gnu.org/onlinedocs/gfortran/ETIME.html
/// [execute_command_line]: https://gcc.gnu.org/onlinedocs/gfortran/EXECUTE_005fCOMMAND_005fLINE.html
/// [exit]: https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html
/// [get_command_argument]: https://gcc.gnu.org/onlinedocs/gfortran/GET_005fCOMMAND_005fARGUMENT.html
/// [getarg]: https://gcc.gnu.org/onlinedocs/gfortran/GETARG.html
/// [iargc]: https://gcc.gnu.org/onlinedocs/gfortran/IARGC.html
/// [lnblnk]: https://gcc.gnu.org/onlinedocs/gfortran/LNBLNK.html
/// [len_trim]: https://gcc.gnu.org/onlinedocs/gfortran/LEN_005fTRIM.html
/// [rand]: https://gcc.gnu.org/onlinedocs/gfortran/RAND.html
/// [random_number]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html
/// [random_seed]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html
/// [srand]: https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html
/// [system]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM.html
/// [system_clock]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM_005fCLOCK.html
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableIntrinsicProcedure {
    /// Extension name
    name: String,
    /// Standard name
    replacement: String,
}

impl Violation for NonPortableIntrinsicProcedure {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, .. } = self;
        format!("Use of non-portable `{name}` call")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { replacement, .. } = self;
        Some(format!("Replace with `{replacement}`"))
    }
}

/// Create a diagnostic with a possible fix that applies given a minimum Fortran standard
fn create_diagnostic(
    context: &CheckContext,
    procedure: &ProcedureCall,
    node: &Node,
    replacement: String,
    fix: Option<Fix>,
    min_std: FortranStandard,
) -> Option<Vec<Diagnostic>> {
    let name = procedure.name().to_string();

    let mut diagnostic =
        context.create_diagnostic(NonPortableIntrinsicProcedure { name, replacement }, node);

    if let Some(fix) = fix {
        let target_std = context.settings().target_std;
        let fix = if target_std < min_std {
            diagnostic.info(format!(
                "A fix is available in Fortran standard {min_std} (target standard: {target_std})"
            ));
            fix.with_applicability(ruff_diagnostics::Applicability::DisplayOnly)
        } else {
            fix
        };
        diagnostic = diagnostic.with_fix(fix);
    }

    some_vec![diagnostic]
}

/// Create diagnostic with a straight-forward replacement
fn create_replacement(
    context: &CheckContext,
    procedure: &ProcedureCall,
    replacement: &str,
    min_std: FortranStandard,
) -> Option<Vec<Diagnostic>> {
    let node = procedure.name().node();
    let edit = node.edit_replacement(replacement.to_string());
    let fix = Fix::unsafe_edit(edit);

    create_diagnostic(
        context,
        procedure,
        node,
        replacement.to_string(),
        Some(fix),
        min_std,
    )
}

/// Create diagnostic with a suggestion but no auto-fix
fn create_non_replacement(
    context: &CheckContext,
    procedure: &ProcedureCall,
    standard: &str,
    min_std: FortranStandard,
) -> Option<Vec<Diagnostic>> {
    let replacement = standard.to_string();
    let name = procedure.name().to_string();

    let mut diagnostic = context.create_diagnostic(
        NonPortableIntrinsicProcedure { name, replacement },
        procedure.name().node(),
    );

    diagnostic.help(format!(
        "The standard `{standard}` has different semantics. See the docs for more details"
    ));

    let target_std = context.settings().target_std;
    if target_std < min_std {
        diagnostic.info(format!(
            "`{standard}` requires Fortran standard {min_std} (target standard: {target_std})"
        ));
    }

    some_vec![diagnostic]
}

/// Create diagnostic with a straight-forward replacement
fn system_replacement(
    context: &CheckContext,
    procedure: &ProcedureCall,
) -> Option<Vec<Diagnostic>> {
    let node = procedure.name().node();
    let replacement = "execute_command_line";
    let edit = node.edit_replacement(replacement.to_string());

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

    create_diagnostic(
        context,
        procedure,
        node,
        replacement.to_string(),
        Some(fix),
        FortranStandard::F2008,
    )
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
    procedure: &ProcedureCall,
) -> Option<Vec<Diagnostic>> {
    let node = procedure.node();
    let is_abort = procedure.name() == "abort";
    let standard = if is_abort { "error stop" } else { "stop" };

    let (replacement, min_std) = stop_replacement(procedure, standard);
    let min_std = if is_abort {
        // `error stop` was added in Fortran 2008
        FortranStandard::F2008.max(min_std)
    } else {
        min_std
    };

    let (fix, replacement) = if let Some(text) = replacement {
        let edit = node.edit_replacement(text.clone());
        (Some(Fix::unsafe_edit(edit)), text)
    } else {
        (None, standard.to_string())
    };

    create_diagnostic(context, procedure, node, replacement, fix, min_std)
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

        match procedure.name().as_lower().as_ref() {
            "system" if is_subroutine => system_replacement(context, &procedure),
            // `system` as a function -- could be replaced by subroutine if not in expression
            "system" => create_non_replacement(
                context,
                &procedure,
                "execute_command_line",
                FortranStandard::F2008,
            ),
            "abort" | "exit" if is_subroutine => create_stop_replacement(context, &procedure),
            "srand" if is_subroutine => {
                create_non_replacement(context, &procedure, "random_seed", FortranStandard::F95)
            }
            "rand" if !is_subroutine => {
                create_non_replacement(context, &procedure, "random_number", FortranStandard::F95)
            }
            "iargc" if !is_subroutine => create_replacement(
                context,
                &procedure,
                "command_argument_count",
                FortranStandard::F2003,
            ),
            // TODO(peter): Intel can have a `status` argument which has a
            // different position in the standard
            "getarg" if is_subroutine => create_replacement(
                context,
                &procedure,
                "get_command_argument",
                FortranStandard::F2003,
            ),
            // This can be subroutine/function, no exact replacement
            "etime" => create_non_replacement(
                context,
                &procedure,
                "cpu_time|system_clock|date_and_time",
                FortranStandard::F95,
            ),
            "idate" | "itime" | "fdate" | "ctime" | "time" | "gmtime" | "ltime" | "time8" => {
                create_non_replacement(context, &procedure, "date_and_time", FortranStandard::F95)
            }
            "lnblnk" => create_replacement(context, &procedure, "len_trim", FortranStandard::F95),
            _ => None,
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression", "subroutine_call"]
    }
}
