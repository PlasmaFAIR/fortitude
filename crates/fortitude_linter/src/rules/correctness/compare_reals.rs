use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use fortitude_sitter::ast::types::Type;
use lazy_regex::regex_is_match;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for equality or inequality comparisons involving real values.
///
/// ## Why is this bad?
/// Real numbers are stored with finite precision, so a value produced by
/// arithmetic may not compare equal to its mathematical result. Comparing real
/// values with `==`, `/=`, `.eq.` or `.ne.` can therefore give surprising
/// results.
///
/// Use an explicit tolerance instead:
///
/// ```f90
/// if (abs(x - y) < tolerance) error stop
/// ```
///
/// ## References
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)
/// - [Ruff RUF069](https://docs.astral.sh/ruff/rules/float-equality-comparison/)
#[derive(ViolationMetadata)]
pub(crate) struct CompareReals {
    operator: String,
}

impl Violation for CompareReals {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { operator } = self;
        format!("real values compared with '{operator}'; use a tolerance instead")
    }
}

impl AstRule for CompareReals {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let operator = node.child_by_field_name("operator")?;
        let operator = operator.text().to_lowercase();
        if !matches!(operator.as_str(), "==" | "/=" | ".eq." | ".ne.") {
            return None;
        }

        let left = node.child_by_field_name("left")?;
        let right = node.child_by_field_name("right")?;
        if !is_real(context, &left) && !is_real(context, &right) {
            return None;
        }

        some_vec![context.create_diagnostic(Self { operator }, node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["relational_expression"]
    }
}

fn is_real(context: &CheckContext, node: &Node) -> bool {
    match node.kind() {
        "number_literal" => {
            regex_is_match!(r"^(\d*\.\d*|\d*\.*\d*[eEdD]-?\d+)(?:_.*)?$", node.text())
        }
        "identifier" => context
            .symbol_table()
            .get_var(node.text())
            .is_some_and(|variable| match variable.type_() {
                Type::Intrinsic(type_) => type_.is_real() || type_.is_double_precision(),
                _ => false,
            }),
        _ => false,
    }
}
