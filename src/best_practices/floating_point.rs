use crate::parser::fortran_language;
use crate::rules::{Code, Violation};
/// Defines rules that ensure floating point types are set in a portable fashion, and
/// checks for potential loss of precision in number literals.
use tree_sitter::{Node, Query};

pub const AVOID_DOUBLE_PRECISION: &str = "\
    The 'double precision' type specifier does not guarantee a 64-bit floating point,
    as one might expect. It is simply required to be twice the size of a default 'real',
    which may vary depending on your system. It may also be modified by compiler
    arguments. For consistency, it is recommended to use `real(dp)`, with `dp` set in
    one of the following ways:

    - `integer, parameter :: dp = selected_real_kind(15, 307)` (Recommended)
    - `use, intrinsic :: iso_fortran_env, only: dp => real64`

    For code that should be compatible with C, you should instead use `real(c_double)`,
    which may be found in the intrinsic module `iso_c_binding`.";

const AVOID_DOUBLE_PRECISION_ERR_MSG: &str = "Instead of 'double precision', use 'real(dp)', \
    with either 'integer, parameter :: dp = selected_real_kind(15, 307)' or \
    'use, intrinsic :: iso_fortran_env, only: dp => real64'";

pub fn avoid_double_precision(code: Code, root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for query_type in ["function_statement", "variable_declaration"] {
        let query_txt = format!("({} (intrinsic_type) @type)", query_type,);
        let query = Query::new(fortran_language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for captures in cursor
            .matches(&query, *root, src.as_bytes())
            .map(|x| x.captures)
        {
            for capture in captures {
                let txt = capture.node.utf8_text(src.as_bytes());
                match txt {
                    Ok(x) => {
                        if x.to_lowercase() == "double precision" {
                            violations.push(Violation::from_node(
                                &capture.node,
                                code,
                                AVOID_DOUBLE_PRECISION_ERR_MSG,
                            ));
                        }
                    }
                    Err(_) => {
                        // Skip, this should be caught by a different rule
                    }
                }
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::{test_tree_method, TEST_CODE};

    #[test]
    fn test_double_precision() {
        let source = "
            double precision function double(x)
              double precision, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              double precision, intent(inout) :: x
              x = 3 * x
            end subroutine

            function quad(x)
              double precision, intent(in) :: x
              double precision :: quad
              quad = 4 * x
            end function
            ";
        let expected_violations = [2, 3, 8, 13, 14]
            .iter()
            .map(|line| Violation::new(*line, TEST_CODE, AVOID_DOUBLE_PRECISION_ERR_MSG))
            .collect();
        test_tree_method(avoid_double_precision, source, Some(expected_violations));
    }
}
