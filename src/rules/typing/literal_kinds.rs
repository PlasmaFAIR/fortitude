use crate::parsing::{child_with_name, dtype_is_number, intrinsic_type, to_text};
use crate::{Method, Rule, Violation};
use lazy_regex::regex_is_match;
use tree_sitter::Node;
/// Defines rules that discourage the use of raw number literals as kinds, as this can result in
/// non-portable code.

// TODO rules for intrinsic kinds in real(x, [KIND]) and similar type casting functions

fn literal_kind(node: &Node, src: &str) -> Option<Violation> {
    let dtype = intrinsic_type(node)?;
    if dtype_is_number(dtype.as_str()) {
        if let Some(child) = child_with_name(node, "size") {
            let txt = to_text(&child, src)?;
            // Match for numbers that aren't preceeded by:
            // - Letters: don't want to catch things like real64
            // - Other numbers: again, shouldn't catch real64
            // - Underscores: could be part of parameter name
            // - "*": 'star kinds' are caught by a different rule
            if regex_is_match!(r"[^A-z0-9_\*]\d", txt) {
                let msg = format!(
                    "{} kind set with number literal, use 'iso_fortran_env' parameter",
                    dtype,
                );
                return Some(Violation::from_node(&msg, node));
            }
        }
    }
    None
}

pub struct LiteralKind {}

impl Rule for LiteralKind {
    fn method(&self) -> Method {
        Method::Tree(literal_kind)
    }

    fn explain(&self) -> &str {
        "
        Rather than setting an intrinsic type's kind using an integer literal, such as
        `real(8)` or `integer(kind=4)`, consider setting kinds using parameters in the
        intrinsic module `iso_fortran_env` such as `real64` and `int32`. For
        C-compatible types, consider instead `iso_c_binding` types such as
        `real(c_double)`.

        Although it is widely believed that `real(8)` represents an 8-byte floating
        point (and indeed, this is the case for most compilers and architectures),
        there is nothing in the standard to mandate this, and compiler vendors are free
        to choose any mapping between kind numbers and machine precision. This may lead
        to surprising results if your code is ported to another machine or compiler.

        For floating point variables, we recommended using `real(sp)` (single
        precision), `real(dp)` (double precision), and `real(qp)` (quadruple precision),
        using:

        ```
        use, intrinsic :: iso_fortran_env, only: sp => real32, &
                                                 dp => real64, &
                                                 qp => real128
        ```

        Or alternatively:

        ```
        integer, parameter :: sp = selected_real_kind(6, 37)
        integer, parameter :: dp = selected_real_kind(15, 307)
        integer, parameter :: qp = selected_real_kind(33, 4931)
        ```

        Some prefer to set one precision parameter `wp` (working precision), which is
        set in one module and used throughout a project.

        Integer sizes may be set similarly:

        ```
        integer, parameter :: i1 = selected_int_kind(2)  ! 8 bits
        integer, parameter :: i2 = selected_int_kind(4)  ! 16 bits
        integer, parameter :: i4 = selected_int_kind(9)  ! 32 bits
        integer, parameter :: i8 = selected_int_kind(18) ! 64 bits
        ```

        Or:

        ```
        use, intrinsic :: iso_fortran_env, only: i1 => int8, &
                                                 i2 => int16, &
                                                 i4 => int32, &
                                                 i8 => int64
        ```
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
        vec!["variable_declaration", "function_statement"]
    }
}

fn literal_kind_suffix(node: &Node, src: &str) -> Option<Violation> {
    let txt = to_text(node, src)?;
    if regex_is_match!(r"_\d+$", txt) {
        let msg = format!(
            "{} has literal suffix, use 'iso_fortran_env' parameter",
            txt,
        );
        return Some(Violation::from_node(&msg, node));
    }
    None
}

pub struct LiteralKindSuffix {}

impl Rule for LiteralKindSuffix {
    fn method(&self) -> Method {
        Method::Tree(literal_kind_suffix)
    }

    fn explain(&self) -> &str {
        "
        Using an integer literal as a kind specifier gives no guarantees regarding the
        precision of the type, as kind numbers are not specified in the Fortran
        standards. It is recommended to use parameter types from `iso_fortran_env`:

        ```
        use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
        ```

        or alternatively:

        ```
        integer, parameter :: sp => selected_real_kind(6, 37)
        integer, parameter :: dp => selected_real_kind(15, 307)
        ```

        Floating point constants can then be specified as follows:

        ```
        real(sp), parameter :: sqrt2 = 1.41421_sp
        real(dp), parameter :: pi = 3.14159265358979_dp
        ```
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
    fn test_literal_kind() -> Result<(), String> {
        let source = dedent(
            "
            integer(8) function add_if(x, y, z)
              integer :: w
              integer(kind=2), intent(in) :: x
              integer(i32), intent(in) :: y
              logical(kind=4), intent(in) :: z

              if (x) then
                add_if = x + y
              else
                add_if = x
              end if
            end function

            subroutine complex_mul(x, y)
              real(8), intent(in) :: x
              complex(4), intent(inout) :: y
              real :: z = 0.5
              y = y * x
            end subroutine

            complex(real64) function complex_add(x, y)
              real(real64), intent(in) :: x
              complex(kind=4), intent(in) :: y
              complex_add = y + x
            end function
            ",
        );
        let expected_violations = [
            (2, 1, "integer"),
            (4, 3, "integer"),
            (6, 3, "logical"),
            (16, 3, "real"),
            (17, 3, "complex"),
            (24, 3, "complex"),
        ]
        .iter()
        .map(|(line, col, kind)| {
            let msg = format!(
                "{} kind set with number literal, use 'iso_fortran_env' parameter",
                kind,
            );
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(&LiteralKind {}, source, Some(expected_violations))?;
        Ok(())
    }

    #[test]
    fn test_literal_kind_suffix() -> Result<(), String> {
        let source = dedent(
            "
            use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

            real(sp), parameter :: x1 = 1.234567_4
            real(dp), parameter :: x2 = 1.234567_dp
            real(dp), parameter :: x3 = 1.789d3
            real(dp), parameter :: x4 = 9.876_8
            real(sp), parameter :: x5 = 2.468_sp
            ",
        );
        let expected_violations = [(4, 29, "1.234567_4"), (7, 29, "9.876_8")]
            .iter()
            .map(|(line, col, num)| {
                let msg = format!(
                    "{} has literal suffix, use 'iso_fortran_env' parameter",
                    num,
                );
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(&LiteralKindSuffix {}, source, Some(expected_violations))?;
        Ok(())
    }
}
