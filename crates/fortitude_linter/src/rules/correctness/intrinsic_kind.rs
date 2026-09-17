use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks calls to certain intrinsic functions that return numbers for a missing
/// explicit `kind` argument.
///
/// ## Why is this bad?
/// Without an explicit `kind` argument, conversions done by the `CMPLX`, `REAL`, `INT`,
/// `AINT`, `ANINT`, `CEILING`, and `FLOOR` intrinsics use a compiler-dependent default
/// kind for their return value. That can silently reduce precision or change the integer
/// kind used by a conversion, which can lead to unexpected results and potentially
/// non-portable behavior.
///
/// ## Example
/// In the following example, While `x` and `y` are declared as real64 variables, the
/// `REAL` intrinsic will return a real32 value when called without an explicit `kind`
/// argument on many compilers, silently truncating the value of `x` and losing precision
/// when it is assigned to `y`.
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: dp => real64, i8 => int64
///
/// real(dp) :: x, y
///
/// x = 1e-10_dp
/// y = real(x)
/// print *, int(y)
/// ```
///
/// Use instead:
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: dp => real64, i8 => int64
///
/// real(dp) :: x, y
///
/// x = 1e-10_dp
/// y = real(x, kind=dp)
/// print *, int(y, kind=i8)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MissingKindArgument {
    intrinsic: String,
}

impl Violation for MissingKindArgument {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { intrinsic } = self;
        format!("'{intrinsic}' call missing 'kind' argument")
    }
}

static REQUIRE_KIND_INTRINSICS: &[&str] =
    &["cmplx", "real", "int", "aint", "anint", "ceiling", "floor"];

impl AstRule for MissingKindArgument {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let name_node = node.child_with_name("identifier")?;
        let intrinsic = name_node.text();
        let intrinsic_lower = intrinsic.to_ascii_lowercase();

        if REQUIRE_KIND_INTRINSICS.contains(&intrinsic_lower.as_str())
            && node
                .child_with_id(kind!("argument_list"))?
                .kwarg("kind")
                .is_none()
        {
            let intrinsic = intrinsic.to_string();
            return some_vec![
                context.create_diagnostic(MissingKindArgument { intrinsic }, name_node)
            ];
        }

        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression"]
    }
}
