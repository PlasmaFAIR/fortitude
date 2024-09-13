use crate::{Method, Rule, Violation};
use lazy_regex::regex_is_match;
use tree_sitter::{Node, Query};
/// Defines rules that ensure real precision is always explicit and stated in a portable way.

// TODO rule to prefer 1.23e4_sp over 1.23e4, and 1.23e4_dp over 1.23d4

fn double_precision_err_msg(dtype: &str) -> Option<String> {
    let lower = dtype.to_lowercase();
    match lower.as_str() {
        "double precision" => Some(String::from(
            "Prefer 'real(real64)' to 'double precision' (see 'iso_fortran_env')",
        )),
        "double complex" => Some(String::from(
            "Prefer 'complex(real64)' to 'double complex' (see 'iso_fortran_env')",
        )),
        _ => None,
    }
}

fn double_precision(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for query_type in ["function_statement", "variable_declaration"] {
        let query_txt = format!("({} (intrinsic_type) @type)", query_type);
        let query = Query::new(&tree_sitter_fortran::language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                let txt = capture.node.utf8_text(src.as_bytes());
                match txt {
                    Ok(x) => {
                        match double_precision_err_msg(x) {
                            Some(msg) => {
                                violations.push(Violation::from_node(&msg, &capture.node));
                            }
                            None => {
                                // Do nothing, found some other intrinsic type
                            }
                        }
                    }
                    Err(_) => {
                        // Skip, non-utf8 text should be caught by a different rule
                    }
                }
            }
        }
    }
    violations
}

pub struct DoublePrecision {}

impl Rule for DoublePrecision {
    fn method(&self) -> Method {
        Method::Tree(double_precision)
    }

    fn explain(&self) -> &str {
        "
        The 'double precision' type does not guarantee a 64-bit floating point number
        as one might expect. It is instead required to be twice the size of a default
        'real', which may vary depending on your system and can be modified by compiler
        arguments. For portability, it is recommended to use `real(dp)`, with `dp` set
        in one of the following ways:

        - `use, intrinsic :: iso_fortran_env, only: dp => real64`
        - `integer, parameter :: dp = selected_real_kind(15, 307)`

        For code that should be compatible with C, you should instead use
        `real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
        "
    }
}

pub struct NoRealSuffix {}

fn no_real_suffix(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let query_txt = "(number_literal) @num";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let txt = capture.node.utf8_text(src.as_bytes());
            match txt {
                Ok(x) => {
                    // Given a number literal, match anything with a decimal place, some amount of
                    // digits either side, and no suffix. This will not catch exponentiation. Tree
                    // sitter will also not include a + or - prefix within the number literal,
                    // considering this to be a unary operator.
                    if regex_is_match!(r"^\d*\.\d*$", x) {
                        let msg = format!(
                            "Floating point literal {} specified without a kind suffix",
                            x,
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

        ! Error, pi is truncated to 6 decimal places
        real(dp), parameter :: pi = 3.14159265358979
        ! Correct
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
    fn test_double_precision() {
        let source = dedent(
            "
            double precision function double(x)
              double precision, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              double precision, intent(inout) :: x
              x = 3 * x
            end subroutine

            function complex_mul(x, y)
              double precision, intent(in) :: x
              double complex, intent(in) :: y
              double complex :: complex_mul
              complex_mul = x * y
            end function
            ",
        );
        let expected_violations = [
            (2, 1, "double precision"),
            (3, 3, "double precision"),
            (8, 3, "double precision"),
            (13, 3, "double precision"),
            (14, 3, "double complex"),
            (15, 3, "double complex"),
        ]
        .iter()
        .map(|(line, col, kind)| {
            let msg = double_precision_err_msg(kind).unwrap();
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(double_precision, source, Some(expected_violations));
    }

    #[test]
    fn test_no_real_suffix() {
        let source = dedent(
            "
            use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

            real(sp), parameter :: x1 = 1.234567
            real(dp), parameter :: x2 = 1.234567_dp
            real(dp), parameter :: x3 = 1.789d3
            real(dp), parameter :: x4 = 9.876
            real(sp), parameter :: x5 = 2.468_sp
            ",
        );
        let expected_violations = [(4, 29, "1.234567"), (7, 29, "9.876")]
            .iter()
            .map(|(line, col, num)| {
                let msg = format!(
                    "Floating point literal {} specified without a kind suffix",
                    num,
                );
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(no_real_suffix, &source, Some(expected_violations));
    }
}
