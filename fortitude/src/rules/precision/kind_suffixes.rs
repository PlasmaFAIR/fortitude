use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use lazy_regex::regex_is_match;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for floating point literal constants that don't have their kinds
/// explicitly specified.
///
/// ## Why is this bad?
/// Floating point literals use the default 'real' kind unless given an explicit
/// kind suffix. This can cause surprising loss of precision:
///
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: dp => real64
///
/// real(dp), parameter :: pi_1 = 3.14159265358979
/// real(dp), parameter :: pi_2 = 3.14159265358979_dp
///
/// print *, pi_1  ! Gives: 3.1415927410125732
/// print *, pi_2  ! Gives: 3.1415926535897900
/// ```
///
/// There are cases where the difference in precision doesn't matter, such
/// as:
///
/// ```f90
/// real(dp) :: x, y
///
/// x = 1.0
/// y = real(2.0, kind=dp)
/// ```
///
/// A case where a missing suffix may be intentional is when using a `kind`
/// statement:
///
/// ```f90
/// integer, parameter :: sp = kind(0.0)
/// ```
///
/// This rule will try to avoid catching these case. However, even for 'nice'
/// numbers, it's possible to accidentally lose precision in surprising ways:
///
/// ```f90
/// real(dp) :: x
///
/// x = sqrt(2.0)
/// ```
///
/// This rule will therefore require an explicit kind statement in the majority
/// of cases where a floating point literal is found in an expression.
///
/// ## References
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/en/learn/best_practices/floating_point/)
#[violation]
pub struct NoRealSuffix {
    literal: String,
}

impl Violation for NoRealSuffix {
    #[derive_message_formats]
    fn message(&self) -> String {
        let NoRealSuffix { literal } = self;
        format!("real literal {literal} missing kind suffix")
    }
}

impl AstRule for NoRealSuffix {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // Given a number literal, match anything with one or more of a decimal place or
        // an exponentiation e or E. There should not be an underscore present.
        // Exponentiation with d or D are ignored, and should be handled with a different
        // rule.
        let txt = node.to_text(src.source_text())?;
        if !regex_is_match!(r"^(\d*\.\d*|\d*\.*\d*[eE]\d+)$", txt) {
            return None;
        }

        // Determine the immediate context in which we've found the literal.
        let mut parent = node.parent()?;
        while matches!(
            parent.kind(),
            "unary_expression" | "parenthesized_expression" | "complex_literal"
        ) {
            parent = parent.parent()?;
        }
        let grandparent = parent.parent()?;

        // Check for loss of precision
        // FIXME: This precision loss test isn't the most reliable
        let value_64: f64 = txt.parse().ok()?;
        let value_32: f32 = txt.parse().ok()?;
        let no_loss = value_64 == 0.0
            || (((value_32 as f64) - value_64) / value_64).abs() < 2.0 * f64::EPSILON;

        // Ok if being used in a direct assignment, provided no loss of precision
        // can occur.
        if matches!(parent.kind(), "assignment_statement" | "init_declarator") && no_loss {
            return None;
        }

        // Ok if being used in a kind statement or a type cast.
        // In the latter case, warnings should still be raised if precision would be
        // lost.
        // If it's the sole argument in a function call, the first parent must be
        // "argument_list", and the second must be "call_expression".
        if grandparent.kind() == "call_expression" {
            if let Some(identifier) = grandparent.child_with_name("identifier") {
                let name = identifier.to_text(src.source_text())?.to_lowercase();
                if name == "kind"
                    || (no_loss
                        && matches!(name.as_str(), "real" | "cmplx" | "dbl" | "int" | "logical"))
                {
                    return None;
                }
            }
        }

        let literal = txt.to_string();
        some_vec![Diagnostic::from_node(NoRealSuffix { literal }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["number_literal"]
    }
}
