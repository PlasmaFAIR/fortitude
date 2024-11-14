use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use lazy_regex::regex_is_match;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Defines rule to ensure real precision is explicit, as this avoids accidental loss of precision.
/// Floating point literals use the default 'real' kind unless given an explicit
/// kind suffix. This can cause surprising loss of precision:
///
/// ```fortran
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
/// ```fortran
/// real(dp) :: x, y
///
/// x = 1.0
/// x = 10.0 * y
/// ```
///
/// However, even for 'nice' numbers, it's possible to accidentally lose
/// precision in surprising ways:
///
/// ```fortran
/// x = y * sqrt(2.0)
/// ```
///
/// Ideally this rule should check how the number is used in a local expression
/// and determine whether precision loss is a real risk, but in its current
/// implementation it instead requires all real literals to have an explicit
/// kind suffix.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_no_real_suffix() -> anyhow::Result<()> {
        let source = test_file(
            "
            use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

            real(sp), parameter :: x1 = 1.234567
            real(dp), parameter :: x2 = 1.234567_dp
            real(dp), parameter :: x3 = 1.789d3 ! rule should ignore d exponentiation
            real(dp), parameter :: x4 = 9.876
            real(sp), parameter :: x5 = 2.468_sp
            real(sp), parameter :: x6 = 2.
            real(sp), parameter :: x7 = .0
            real(sp), parameter :: x8 = 1E2
            real(sp), parameter :: x9 = .1e2
            real(sp), parameter :: y1 = 1.E2
            real(sp), parameter :: y2 = 1.2e3

            ",
        );
        let expected: Vec<_> = [
            (3, 28, 3, 36, "1.234567"),
            (6, 28, 6, 33, "9.876"),
            (8, 28, 8, 30, "2."),
            (9, 28, 9, 30, ".0"),
            (10, 28, 10, 31, "1E2"),
            (11, 28, 11, 32, ".1e2"),
            (12, 28, 12, 32, "1.E2"),
            (13, 28, 13, 33, "1.2e3"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, num)| {
            Diagnostic::from_start_end_line_col(
                NoRealSuffix {
                    literal: num.to_string(),
                },
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let actual = NoRealSuffix::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
