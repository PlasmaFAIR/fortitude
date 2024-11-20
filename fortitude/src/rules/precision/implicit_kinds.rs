use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for `real` variables that don't have their kind explicitly specified.
///
/// ## Why is this bad?
/// Real variable declarations without an explicit kind will have a compiler/platform
/// dependent precision, which hurts portability and may lead to surprising loss of
/// precision in some cases. Although the default `real` will map to a 32-bit floating
/// point number on most systems, this is not guaranteed.
///
/// It is recommended to always be explicit about the precision required by `real`
/// variables. This can be done by setting their 'kinds' using integer parameters
/// chosen in one of the following ways:
///
/// ```f90
/// ! Set using iso_fortran_env
/// use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
/// ! Using selected_real_kind
/// integer, parameter :: sp = selected_real_kind(6, 37)
/// integer, parameter :: dp = selected_real_kind(15, 307)
/// ! For C-compatibility:
/// use, intrinsic :: iso_c_binding, only: sp => c_float, dp => c_double
///
/// ! Declaring real variables:
/// real(sp) :: single
/// real(dp) :: double
/// ```
///
/// It is also common for Fortran developers to set a 'working precision' `wp`,
/// which is set to either `sp` or `dp` and used throughout a project. This can
/// then be easily toggled depending on the user's needs.
///
/// ## References
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/en/learn/best_practices/floating_point/)
#[violation]
pub struct ImplicitRealKind {
    dtype: String,
}

impl Violation for ImplicitRealKind {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ImplicitRealKind { dtype } = self;
        format!("{dtype} has implicit kind")
    }
}

impl AstRule for ImplicitRealKind {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let dtype = node.child(0)?.to_text(src.source_text())?.to_lowercase();

        if !matches!(dtype.as_str(), "real" | "complex") {
            return None;
        }

        if node.child_by_field_name("kind").is_some() {
            return None;
        }

        some_vec![Diagnostic::from_node(ImplicitRealKind { dtype }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
