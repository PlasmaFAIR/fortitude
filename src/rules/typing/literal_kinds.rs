use crate::{Method, Rule, Violation};
use lazy_regex::regex_is_match;
use tree_sitter::{Node, Query};
/// Defines rules that discourage the use of raw number literals as kinds, as this can result in
/// non-portable code.

fn literal_kind_err_msg(dtype: &str) -> Option<String> {
    let lower = dtype.to_lowercase();
    match lower.as_str() {
        "integer" | "logical" => Some(format!(
            "Avoid setting {} kind with raw number literals, use a parameter from \
            'iso_fortran_env' or set using 'selected_int_kind'",
            lower,
        )),
        "real" | "complex" => Some(format!(
            "Avoid setting {} kind with raw number literals, use a parameter from \
            'iso_fortran_env' or set using 'selected_real_kind'",
            lower,
        )),
        _ => None,
    }
}

fn literal_kind(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for query_type in ["function_statement", "variable_declaration"] {
        // Find intrinstic types annotated with a single number literal, or with a
        // 'kind' keyword. This will also pick up characters with a a length specified,
        // but we'll skip those later.
        let query_txt = format!(
            "
            ({}
                (intrinsic_type) @type
                (size
                    [
                        (argument_list (number_literal))
                        (argument_list
                            (keyword_argument
                                name: (identifier)
                                value: (number_literal)
                            )
                        )
                    ]
                )
            )",
            query_type,
        );
        let query = Query::new(&tree_sitter_fortran::language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                let txt = capture.node.utf8_text(src.as_bytes());
                match txt {
                    Ok(x) => {
                        match literal_kind_err_msg(x) {
                            Some(msg) => {
                                violations.push(Violation::from_node(&msg, &capture.node));
                            }
                            None => {
                                // Do nothing, characters should be handled elsewhere
                            }
                        }
                    }
                    Err(_) => {
                        // Skip, non utf8 text should be caught by a different rule
                    }
                }
            }
        }
    }
    violations
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
}

fn literal_kind_suffix(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    // Given a number literal, match anything suffixed with plain number.
    // TODO Match either int or real, change error message accordingly

    let query_txt = "(number_literal) @num";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let txt = capture.node.utf8_text(src.as_bytes());
            match txt {
                Ok(x) => {
                    if regex_is_match!(r"_\d+$", x) {
                        let msg = format!(
                            "Instead of number literal suffix in {}, use parameter suffix \
                            from 'iso_fortran_env'",
                            x
                        );
                        violations.push(Violation::from_node(&msg, &capture.node));
                    }
                }
                Err(_) => {
                    // Skip, non-utf8 text should be caught by a different rule
                }
            }
        }
    }
    violations
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::test_tree_method;
    use crate::violation;
    use textwrap::dedent;

    #[test]
    fn test_literal_kind() {
        let source = dedent(
            "
            integer(8) function add_if(x, y, z)
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
            (3, 3, "integer"),
            (5, 3, "logical"),
            (15, 3, "real"),
            (16, 3, "complex"),
            (22, 3, "complex"),
        ]
        .iter()
        .map(|(line, col, kind)| {
            let msg = literal_kind_err_msg(kind).unwrap();
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(literal_kind, source, Some(expected_violations));
    }

    #[test]
    fn test_literal_kind_suffix() {
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
                    "Instead of number literal suffix in {}, use parameter suffix from \
                    'iso_fortran_env'",
                    num,
                );
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(literal_kind_suffix, source, Some(expected_violations));
    }
}
