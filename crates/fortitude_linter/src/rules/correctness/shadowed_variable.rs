use crate::CheckContext;
use crate::ast::symbol_table::{Symbol, SymbolTable};
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
/// There are contexts in which shadowing is acceptable. For instance, if a
/// procedure dummy argument has the same name as a module variable, this
/// indicates that the programmer likely intends to use the dummy argument to
/// reference the module variable. However, shadowing a module-scoped variable
/// with a local variable is generally considered poor practice.
///
/// It is also very common to reuse loop variables such as `i`, `j`, and `k` or
/// error flags such as `err` in different scopes. Fortitude will ignore
/// integers with common names. The setting `check.shadowed-variables.allow` can
/// be used to add further variables to the whitelist.
///
/// The setting `check.shadowed-variables.strict` can be used to disallow
/// shadowing of variables in all contexts, including dummy arguments.
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
/// ```
///
/// ## Settings
/// See [check.shadowed-variables](../settings.md#checkshadowed-variables)
#[derive(ViolationMetadata)]
pub struct ShadowedVariable {
    name: String,
}

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
    let allow = &context.settings().shadowed_variables.allow;
    let strict = &context.settings().shadowed_variables.strict;
    // Check the names of all variables and used items declared in the current scope
    let mut diagnostics = symbols
        .iter()
        // Only check variables and used items
        .filter(|(_, symbol)| symbol.is_variable() || symbol.is_used_item())
        // Filter dummy variables and allowed names
        .filter(|(name, var)| {
            if allow.contains(name) {
                return false;
            }
            if let Symbol::Variable(var) = var {
                return *strict || !var.is_dummy_var();
            }
            true
        })
        // Create diagnostic if var found in a higher scope
        .filter_map(|(name, var)| {
            if let Some(parent) = context.symbol_table().get(name)
                && (parent.is_variable() || parent.is_used_item())
            {
                Some(context.create_diagnostic(
                    ShadowedVariable {
                        name: name.to_string(),
                    },
                    var.name(),
                ))
            } else {
                None
            }
        })
        .collect_vec();
    diagnostics.sort();
    diagnostics
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::Display;

    #[derive(Debug, Clone, CacheKey)]
    pub struct Settings {
        pub allow: Vec<String>,
        pub strict: bool,
    }

    const DEFAULT_ALLOWED: &[&str] = &[
        "i", "j", "k", "l", "m", "n", "ii", "jj", "kk", "ll", "mm", "nn", "idx", "index", "err",
        "ierr", "ioerr", "ios", "info", "stat", "iostat", "istat", "status",
    ];

    impl Default for Settings {
        fn default() -> Self {
            Self {
                allow: DEFAULT_ALLOWED.iter().map(|s| s.to_string()).collect(),
                strict: false,
            }
        }
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.shadowed-variables",
                fields = [self.allow | array, self.strict]
            }
            Ok(())
        }
    }
}
