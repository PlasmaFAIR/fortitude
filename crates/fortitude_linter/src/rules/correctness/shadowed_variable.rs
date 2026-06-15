use crate::CheckContext;
use crate::ast::symbol_table::{Symbol, SymbolTable};
use crate::ast::types::HasName;
use crate::diagnostics::{Diagnostic, FixAvailability, Violation};
use fortitude_macros::ViolationMetadata;
use itertools::Itertools;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for variables that shadow those in a higher scope.
///
/// ## Why is this bad?
/// Shadowing variables can lead to confusion, as it can be unclear which
/// variable is being referenced in a given context. Shadowing may be
/// unintentional, which is a common source of bugs.
///
/// There are contexts in which shadowing is acceptable. For instance,
/// if a procedure dummy argument has the same name as a module variable, this is acceptable,
/// this indicates that the programmer likely intends to use the dummy argument
/// in preference to the module variable. However, shadowing a module variable
/// with a local variable is generally considered poor practice.
///
/// It is also very common to reuse loop variables such as `i`, `j`, and `k` or error
/// flags such as `err` in different scopes. Fortitude will ignore integers with common
/// names. The setting `shadowed-variables.allow` can be used to add further variables
/// to the whitelist.
///
/// ## Examples
///
/// ```f90
/// module my_mod
///
///   implicit none (type, external)
///   private
///
///   real, allocatable :: x(:)
///
/// contains
///
///   subroutine initialise(n)
///     integer, intent(in) :: n
///     real, allocatable :: x(:)  ! This is a bug!
///
///     allocate(x(n))
///
///   end subroutine initialise
///
///   subroutine selection_sort(arr)
///     real, intent(inout) :: arr(:)
///     integer :: i
///
///     do i = 1, size(arr)
///       call helper(arr(i:size(arr)))
///     end do
///
///   contains
///
///     !! Finds minimum element of an array, swaps it with the start
///     subroutine helper(arr)
///       real, intent(inout) :: arr(:) ! Allowed: is a dummy arg
///       real :: min_val
///       integer :: min_idx
///       integer :: i  ! Allowed: is a loop variable
///
///       min_val = arr(1)
///       min_idx = 1
///       do i = 1, size(arr)
///         if (arr(i) < min_val) then
///           min_val = arr(i)
///           min_idx = i
///         end if
///       end do
///       arr(min_idx) = arr(1)
///       arr(1) = min_val
///
///     end subroutine helper
///
///   end subroutine selection_sort
///
/// end module my_mod
#[derive(ViolationMetadata)]
pub struct ShadowedVariable {
    name: String,
}

// const ALLOWED_INTS: &[&'static str] = &[
//     "i", "j", "k", "ii", "jj", "kk", "idx", "index", "err", "ierr", "ioerr", "stat", "iostat",
//     "status",
// ];

impl Violation for ShadowedVariable {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        let name = &self.name;
        // TODO: Use new diagnostic annotations to point to higher scope variable
        format!("Variable `{name}` shadows variable in a higher scope")
    }
}

/// Check the symbol table for any variables that also exist at a higher scope.
pub(crate) fn check_shadowed_variables(
    context: &CheckContext,
    symbols: &SymbolTable,
) -> Vec<Diagnostic> {
    // Check the names of all variables declared in the current scope
    symbols
        .iter()
        .filter_map(|(name, symbol)| {
            if let Symbol::Variable(symbol) = symbol
                && let Some(_) = context.symbol_table().get_var(name)
            {
                Some(context.create_diagnostic(
                    ShadowedVariable {
                        name: name.to_string(),
                    },
                    symbol.name(),
                ))
            } else {
                None
            }
        })
        .collect_vec()
}
