use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of a unary expression following an arithmetic operator.
///
/// ## Why is this bad?
/// The use of a unary operator (`+`, `-` or user-defined) following an arithmetic operator (`+`,
/// `-`, `*`, `/`, `**` or user-defined) can be ambiguous and is not part of the Fortran standard.
/// The order of operations does not necessarily follow typical mathematical order. Some compilers
/// may warn users of this code smell, but only via extensions. The use of a unary operator
/// following an arithmetic operator may result in unexpected behaviour and/or differences in output
/// between compilers. Use parentheses to remove ambiguity of user expected output.
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
    #[derive_message_formats]
    fn message(&self) -> String {
        "Unary operator following arithmetic operator (use parentheses)".to_string()
    }
}

impl AstRule for UnaryFollowingArithmetic {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let rhs = node.child_by_field_name("right")?;

        if rhs.kind() != "unary_expression" {
            return None;
        }

        let unary_operator = rhs.child(0)?;

        some_vec![context.create_diagnostic(UnaryFollowingArithmetic, unary_operator)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["math_expression"]
    }
}
