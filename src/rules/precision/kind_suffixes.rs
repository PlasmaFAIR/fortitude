use crate::parsing::to_text;
use crate::{Method, Rule, Violation};
use lazy_regex::regex_is_match;
use tree_sitter::Node;
/// Defines rule to ensure real precision is explicit, as this avoids accidental loss of precision.

fn no_real_suffix(node: &Node, src: &str) -> Option<Violation> {
    // Given a number literal, match anything with one or more of a decimal place or
    // an exponentiation e or E. There should not be an underscore present.
    // Exponentiation with d or D are ignored, and should be handled with a different
    // rule.
    let txt = to_text(node, src)?;
    if regex_is_match!(r"^(\d*\.\d*|\d*\.*\d*[eE]\d+)$", txt) {
        let msg = format!("real literal {} has no kind suffix", txt);
        return Some(Violation::from_node(&msg, node));
    }
    None
}

pub struct NoRealSuffix {}

impl Rule for NoRealSuffix {
    fn method(&self) -> Method {
        Method::Tree(no_real_suffix)
    }

    fn explain(&self) -> &str {
        "
        Floating point literals use the default 'real' kind unless given an explicit
        kind suffix. This can cause surprising loss of precision:

        ```
        use, intrinsic :: iso_fortran_env, only: dp => real64

        real(dp), parameter :: pi_1 = 3.14159265358979
        real(dp), parameter :: pi_2 = 3.14159265358979_dp

        print *, pi_1  ! Gives: 3.1415927410125732 
        print *, pi_2  ! Gives: 3.1415926535897900
        ```

        There are many cases where the difference in precision doesn't matter, such
        as the following operations:

        ```
        real(dp) :: x, y

        x = 1.0
        x = 10.0 * y
        ```

        However, even for 'nice' numbers, it's possible to accidentally lose
        precision in surprising ways:

        ```
        x = y * sqrt(2.0)
        ```

        Ideally this rule should check how the number is used in a local expression
        and determine whether precision loss is a real risk, but in its current
        implementation it instead requires all real literals to have an explicit
        kind suffix.
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
        vec!["number_literal"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::test_tree_method;
    use crate::violation;
    use textwrap::dedent;

    #[test]
    fn test_no_real_suffix() -> Result<(), String> {
        let source = dedent(
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
        let expected_violations = [
            (4, 29, "1.234567"),
            (7, 29, "9.876"),
            (9, 29, "2."),
            (10, 29, ".0"),
            (11, 29, "1E2"),
            (12, 29, ".1e2"),
            (13, 29, "1.E2"),
            (14, 29, "1.2e3"),
        ]
        .iter()
        .map(|(line, col, num)| {
            let msg = format!("real literal {} has no kind suffix", num);
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(&NoRealSuffix {}, &source, Some(expected_violations))?;
        Ok(())
    }
}
