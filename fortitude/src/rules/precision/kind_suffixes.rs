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
/// There are many cases where the difference in precision doesn't matter, such
/// as the following operations:
///
/// ```f90
/// real(dp) :: x, y
///
/// x = 1.0
/// x = 10.0 * y
/// ```
///
/// However, even for 'nice' numbers, it's possible to accidentally lose
/// precision in surprising ways:
///
/// ```f90
/// x = y * sqrt(2.0)
/// ```
///
/// Ideally, this rule should check how the number is used in a local expression
/// and determine whether precision loss is a real risk, but in its current
/// implementation it instead requires all real literals to have an explicit
/// kind suffix.
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
        let literal = txt.to_string();
        some_vec![Diagnostic::from_node(NoRealSuffix { literal }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["number_literal"]
    }
}
