use crate::parser::fortran_language;
use crate::rules::{Code, Violation};
/// Defines rules that discourage the use of raw number literals as kinds, and the use
/// of non-standard extensions to the language such as the type `real*8`.
use regex::Regex;
use tree_sitter::{Node, Query};

pub const AVOID_NUMBER_LITERAL_KINDS: &str = "\
    Rather than setting an intrinsic type's kind using an integer literal, such as
    `real(8)` or `integer(kind=4)`, consider setting precision using the built-in
    `selected_real_kind` or `selected_int_kind` functions. Although it is widely 
    believed that `real(8)` represents an 8-byte floating point (and indeed, this is the
    case for most compilers and architectures), there is nothing in the standard to
    mandate this, and compiler vendors are free to choose any mapping between kind
    numbers and machine precision. This may lead to surprising results if your code is
    ported to another machine or compiled with a difference compiler.

    For floating point variables, it is recommended to use `real(sp)` (single 
    precision), `real(dp)` (double precision), and `real(qp)` (quadruple precision), 
    using:

    ```
    integer, parameter :: sp = selected_real_kind(6, 37)
    integer, parameter :: dp = selected_real_kind(15, 307)
    integer, parameter :: qp = selected_real_kind(33, 4931)
    ```

    Or alternatively:

    ```
    use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64, qp => real128
    ```

    Some prefer to set one precision parameter `wp` (working precision), which is set
    in one module and used throughout a project.

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

    For code that should be compatible with C, you should instead set kinds such as 
    `real(c_double)` or `integer(c_int)` which may be found in the intrinsic module 
    `iso_c_binding`.";

fn literal_kind_err_msg(dtype: &str) -> Option<String> {
    let lower = dtype.to_lowercase();
    match lower.as_str() {
        "integer" | "logical" => Some(format!(
            "Avoid setting {} kind with raw number literals, and instead use a parameter set \
                using 'selected_int_kind' or one of `int8`, `int16`, `int32` or `int64` from the \
                `iso_fortran_env` module",
            lower,
        )),
        "real" | "complex" => Some(format!(
            "Avoid setting {} kind with raw number literals, and instead use a parameter set \
                using 'selected_real_kind' or one of `real32`, `real64`, or `real128` from the \
                `iso_fortran_env` module",
            lower,
        )),
        _ => None,
    }
}

pub fn avoid_number_literal_kinds(code: Code, root: &Node, src: &str) -> Vec<Violation> {
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
        let query = Query::new(fortran_language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                let txt = capture.node.utf8_text(src.as_bytes());
                match txt {
                    Ok(x) => {
                        match literal_kind_err_msg(x) {
                            Some(y) => {
                                violations.push(Violation::from_node(
                                    &capture.node,
                                    code,
                                    y.as_str(),
                                ));
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

pub const AVOID_NON_STANDARD_BYTE_SPECIFIER: &str = "\
    Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
    avoided. For these cases, consider instead using 'real(real64)' or 'integer(int32)',
    where 'real64' and 'int32' may be found in the intrinsic module 'iso_fortran_env'.
    You may also wish to determine kinds using the built-in functions
    'selected_real_kind' and 'selected_int_kind'.";


const NON_STANDARD_BYTE_SPECIFIER_ERR: &str = "Avoid non-standard 'type*N', prefer 'type(N)'";

pub fn avoid_non_standard_byte_specifier(code: Code, root: &Node, src: &str) -> Vec<Violation> {
    // Note: This does not match 'character*(*)', which should be handled by a different
    // rule.
    let mut violations = Vec::new();
    // Match anything beginning with a '*' followed by any amount of whitespace or '&'
    // symbols (in case you like to split your type specifiers over multiple lines),
    // followed by at least one digit.
    let re = Regex::new(r"^\*[\s&]*\d+").unwrap();

    for query_type in ["function_statement", "variable_declaration"] {
        let query_txt = format!("({} (intrinsic_type) (size) @size)", query_type);
        let query = Query::new(fortran_language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                match capture.node.utf8_text(src.as_bytes()) {
                    Ok(x) => {
                        if re.is_match(x) {
                            violations.push(Violation::from_node(
                                &capture.node,
                                code,
                                NON_STANDARD_BYTE_SPECIFIER_ERR,
                            ));
                        }
                    }
                    Err(_) => {
                        // Found non utf8 text, should be caught by a different rule,
                        continue;
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
            ";
        let expected_violations = [2, 3, 5, 15, 16, 22]
            .iter()
            .zip([
                "integer", "integer", "logical", "real", "complex", "complex",
            ])
            .map(|(line, kind)| {
                Violation::new(
                    *line,
                    TEST_CODE,
                    literal_kind_err_msg(kind).unwrap().as_str(),
                )
            })
            .collect();
        test_tree_method(
            avoid_number_literal_kinds,
            source,
            Some(expected_violations),
        );
    }

    #[test]
    fn test_non_standard_byte_specifier() {
        let source = "
            integer*8 function add_if(x, y, z)
              integer(kind=2), intent(in) :: x
              integer *4, intent(in) :: y
              logical*   4, intent(in) :: z

              if (x) then
                add_if = x + y
              else
                add_if = x
              end if
            end function

            subroutine complex_mul(x, y)
              real * 8, intent(in) :: x
              complex  *  16, intent(inout) :: y
              y = y * x
            end subroutine
            ";
        let expected_violations = [2, 4, 5, 15, 16]
            .iter()
            .map(|line| {
                Violation::new(
                    *line,
                    TEST_CODE,
                    NON_STANDARD_BYTE_SPECIFIER_ERR,
                )
            })
            .collect();
        test_tree_method(
            avoid_non_standard_byte_specifier,
            source,
            Some(expected_violations),
        );
    }
}
