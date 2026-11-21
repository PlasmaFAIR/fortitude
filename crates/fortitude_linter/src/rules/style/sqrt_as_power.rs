use crate::diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for square roots written as a power of one half, such as `x ** 0.5`.
///
/// ## Why is this bad?
/// `sqrt(x)` states the intent directly, and is typically evaluated using a
/// dedicated (and often faster) square root instruction, while `x ** 0.5` may
/// be evaluated by a general-purpose power function.
///
/// ## Automatic Fix
/// The automatic fix is unsafe, as `sqrt(x)` and `x ** 0.5` may not give
/// bit-identical results, and may differ in how they handle special values
/// such as negative zero.
///
/// ## Example
/// ```f90
/// y = x ** 0.5
/// ```
///
/// Use instead:
/// ```f90
/// y = sqrt(x)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SqrtAsPower;

impl Violation for SqrtAsPower {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "`x ** 0.5` can be written as `sqrt(x)`".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `sqrt`".to_string())
    }
}

/// Is this node a real literal equal to 0.5, possibly in parentheses?
fn is_half(node: &Node) -> bool {
    let mut node = *node;
    while node.kind() == "parenthesized_expression" {
        match node.named_child(0) {
            Some(inner) => node = inner,
            None => return false,
        }
    }
    if node.kind() != "number_literal" {
        return false;
    }
    // Drop the kind suffix (`0.5_dp`) and allow `d` exponents (`5d-1`).
    let text = node.text().split('_').next().unwrap_or("").to_lowercase();
    // Integers are excluded: `** 5` is not `** 0.5`.
    text.contains(['.', 'e', 'd']) && text.replace('d', "e").parse::<f64>() == Ok(0.5)
}

impl AstRule for SqrtAsPower {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.child(1)?.text() != "**" || !is_half(&node.child_by_field_name("right")?) {
            return None;
        }
        let left = node.child_by_field_name("left")?;
        // `(a + b) ** 0.5` becomes `sqrt(a + b)`, not `sqrt((a + b))`.
        let arg = match left.kind() {
            "parenthesized_expression" => left.named_child(0)?,
            _ => left,
        };
        let fix = Fix::unsafe_edit(node.edit_replacement(format!("sqrt({})", arg.text())));
        some_vec![context.create_diagnostic(Self, node).with_fix(fix)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["math_expression"]
    }
}
