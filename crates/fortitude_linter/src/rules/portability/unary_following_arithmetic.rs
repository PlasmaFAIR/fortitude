use crate::diagnostics::{Diagnostic, FixAvailability, Violation};
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_diagnostics::{Edit, Fix};
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of a unary expression following an arithmetic operator.
///
/// ## Why is this bad?
/// The use of a unary operator (`+`, `-` but *not* user-defined) following an arithmetic operator (
/// `+`, `-`, `*`, `/`, `**` but *not* user-defined) can be ambiguous and is not part of the Fortran
/// standard. The order of operations does not necessarily follow typical mathematical order. See
/// (the Doctor Fortran
/// article)[https://stevelionel.com/drfortran/2021/04/03/doctor-fortran-in-order-order/].
///
/// Some compilers may warn users of this code smell, but only via extensions. The use of a unary
/// operator following an arithmetic operator may result in unexpected behaviour and/or differences
/// in output between compilers. Use parentheses to remove ambiguity of user expected output; the
/// fix may change prior behaviour on some compilers.
///
/// ## Example
/// ```f90
/// x = 10 ** -2 * 2
/// ! Would expected x = 0.02 but some compilers may give x = 0.0001.
/// ```
///
/// Use instead:
/// ```f90
/// x = 10 ** (-2) * 2
/// ! Result is unambiguously x = 0.02.
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UnaryFollowingArithmetic;

impl Violation for UnaryFollowingArithmetic {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Unary operator following an arithmetic expression".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add parenthesese around the desired argument of the unary operator".to_string())
    }
}

impl AstRule for UnaryFollowingArithmetic {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // Check if we have a unary expression on the RHS of this math expression.
        let unary_exp = node.child_by_field_name("right")?;

        if unary_exp.kind() != "unary_expression" {
            return None;
        }

        // User defined unary operators have precedence so don't cause problems.
        let unary_op = unary_exp.child_by_field_name("operator")?;

        if unary_op.kind() == "user_defined_operator" {
            return None;
        }

        // If for some strange reason we have multiple unary operators, e.g `10 ** -+2`, then we
        // need the right-most one so its argument can be the only thing parenthesised.
        let unary_arg = right_most_unary(unary_exp).child_by_field_name("argument")?;

        let parenthesise_end_node = if unary_arg.kind() == "math_expression" {
            unary_arg.child_by_field_name("left")?
        } else {
            unary_arg
        };

        // All fixes are safe except for those of the form `a ** -b * c` where the `-` can also be a
        // `+` and the `*` can also be a `/`.
        let fix = if is_unsafe_fix(node, &unary_arg, &unary_op) {
            Fix::unsafe_edits(
                Edit::insertion("(".to_string(), unary_exp.start_textsize()),
                [Edit::insertion(
                    ")".to_string(),
                    parenthesise_end_node.end_textsize(),
                )],
            )
        } else {
            Fix::safe_edits(
                Edit::insertion("(".to_string(), unary_exp.start_textsize()),
                [Edit::insertion(
                    ")".to_string(),
                    parenthesise_end_node.end_textsize(),
                )],
            )
        };

        some_vec![
            context
                .create_diagnostic(UnaryFollowingArithmetic, unary_op)
                .with_fix(fix)
        ]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["math_expression"]
    }
}

fn is_unsafe_fix(node: &Node, unary_arg: &Node, unary_op: &Node) -> bool {
    // An unsafe fix is when we have something of the form `a ** -b * c` where the `-` can also be a
    // `+` and the `*` can also be a `/`.

    let Some(node_op) = node.child_by_field_name("operator") else {
        return false;
    };

    if node_op.kind() != "**" {
        return false;
    }

    // "-" or "+" part.
    if unary_op.kind() != "-" && unary_op.kind() != "+" {
        return false;
    }

    // "*" or "/" part.
    if unary_arg.kind() != "math_expression" {
        return false;
    }

    let Some(unary_arg_op) = unary_arg.child_by_field_name("operator") else {
        return false;
    };

    matches!(unary_arg_op.kind(), "*" | "/")
}

fn right_most_unary(mut unary_exp: Node) -> Node {
    // Recursively finds the right-most unary operator.
    // E.g `-x` -> gives the `-`.
    // E.g `-+x` -> gives the `+`.
    // E.g `+-+-+-x` -> gives the final `-`.

    loop {
        let Some(unary_arg) = unary_exp.child_by_field_name("argument") else {
            return unary_exp;
        };

        if unary_arg.kind() != "unary_expression" {
            return unary_exp;
        }

        unary_exp = unary_arg;
    }
}
