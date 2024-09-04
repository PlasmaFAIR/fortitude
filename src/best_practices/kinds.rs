use crate::core::{Method, Rule, Violation};
use crate::settings::Settings;
use regex::Regex;
use tree_sitter::{Node, Query};
/// Defines rules that discourage the use of raw number literals as kinds, and the use
/// of non-standard extensions to the language such as the type `real*8`. Also contains
/// rules to ensure floating point types are set in a portable manner, and checks for
/// potential loss of precision in number literals.

// TODO rule to prefer 1.23e4_sp over 1.23e4, and 1.23e4_dp over 1.23d4

// Avoid number literal kinds
// --------------------------

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

fn avoid_number_literal_kinds(root: &Node, src: &str) -> Vec<Violation> {
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

pub struct AvoidNumberLiteralKinds {}

impl Rule for AvoidNumberLiteralKinds {
    fn method(&self) -> Method {
        Method::Tree(Box::new(avoid_number_literal_kinds))
    }

    fn explain(&self) -> &str {
        "
        Rather than setting an intrinsic type's kind using an integer literal, such as
        `real(8)` or `integer(kind=4)`, consider setting precision using the built-in
        `selected_real_kind` or `selected_int_kind` functions. Although it is widely
        believed that `real(8)` represents an 8-byte floating point (and indeed, this is
        the case for most compilers and architectures), there is nothing in the standard
        to mandate this, and compiler vendors are free to choose any mapping between
        kind numbers and machine precision. This may lead to surprising results if your
        code is ported to another machine or compiled with a difference compiler.

        For floating point variables, it is recommended to use `real(sp)` (single
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

        For code that should be compatible with C, you should instead set kinds such as
        `real(c_double)` or `integer(c_int)` which may be found in the intrinsic module
        `iso_c_binding`.
        "
    }
}

// Avoid non-standard bytes specifier
// ----------------------------------

fn avoid_non_standard_byte_specifier(root: &Node, src: &str) -> Vec<Violation> {
    // Note: This does not match 'character*(*)', which should be handled by a different
    // rule.
    let mut violations = Vec::new();
    // Match anything beginning with a '*' followed by any amount of whitespace or '&'
    // symbols (in case you like to split your type specifiers over multiple lines),
    // followed by at least one digit.
    let re = Regex::new(r"^\*[\s&]*\d+").unwrap();

    for query_type in ["function_statement", "variable_declaration"] {
        let query_txt = format!("({} (intrinsic_type) (size) @size)", query_type);
        let query = Query::new(&tree_sitter_fortran::language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                match capture.node.utf8_text(src.as_bytes()) {
                    Ok(x) => {
                        if re.is_match(x) {
                            let msg = "Avoid non-standard 'type*N', prefer 'type(N)'";
                            violations.push(Violation::from_node(msg, &capture.node));
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

pub struct AvoidNonStandardByteSpecifier {}

impl Rule for AvoidNonStandardByteSpecifier {
    fn method(&self) -> Method {
        Method::Tree(Box::new(avoid_non_standard_byte_specifier))
    }

    fn explain(&self) -> &str {
        "
        Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
        avoided. For these cases, consider instead using 'real(real64)' or
        'integer(int32)', where 'real64' and 'int32' may be found in the intrinsic
        module 'iso_fortran_env'. You may also wish to determine kinds using the
        built-in functions 'selected_real_kind' and 'selected_int_kind'.
        "
    }
}

// Avoid double precision type
// ---------------------------

fn double_precision_err_msg(dtype: &str) -> Option<String> {
    let lower = dtype.to_lowercase();
    match lower.as_str() {
        "double precision" => Some(String::from(
            "Instead of 'double precision', use 'real(dp)', with 'dp' set to 'real64' \
            from the intrinsic module 'iso_fortran_env'",
        )),
        "double complex" => Some(String::from(
            "Instead of 'double complex', use 'complex(dp)', with 'dp' set to 'real64' \
            from the intrinsic module 'iso_fortran_env'",
        )),
        _ => None,
    }
}

fn avoid_double_precision(root: &Node, src: &str) -> Vec<Violation> {
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

pub struct AvoidDoublePrecision {}

impl Rule for AvoidDoublePrecision {
    fn method(&self) -> Method {
        Method::Tree(Box::new(avoid_double_precision))
    }

    fn explain(&self) -> &str {
        "
        The 'double precision' type specifier does not guarantee a 64-bit floating
        point, as one might expect. It is simply required to be twice the size of a
        default 'real', which may vary depending on your system and can be modified
        by compiler arguments. For consistency, it is recommended to use `real(dp)`,
        with `dp` set in one of the following ways:

        - `use, intrinsic :: iso_fortran_env, only: dp => real64`
        - `integer, parameter :: dp = selected_real_kind(15, 307)`

        For code that should be compatible with C, you should instead use
        `real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
        "
    }
}

// Use floating point suffixes
// ---------------------------

pub struct UseFloatingPointSuffixes {
    strict: bool,
}

impl UseFloatingPointSuffixes {
    pub fn new(settings: &Settings) -> Self {
        Self {
            strict: settings.strict,
        }
    }

    fn rule(&self, root: &Node, src: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        // Given a number literal, match anything with a decimal place, some amount of
        // digits either side, and no suffix. This will not catch exponentiation.
        // Tree sitter will also not include a + or - prefix within the number literal,
        // considering this to be a unary operator.
        let re = Regex::new(r"^\d*\.\d*$").unwrap();

        let query_txt = "(number_literal) @num";
        let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                let txt = capture.node.utf8_text(src.as_bytes());
                match txt {
                    Ok(x) => {
                        if re.is_match(x) && (self.strict || x.len() > 6) {
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
}

impl Rule for UseFloatingPointSuffixes {
    fn method(&self) -> Method {
        Method::Tree(Box::new(move |root, src| self.rule(root, src)))
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

        Floating point constants should therefore always be specified with a kind
        suffix.

        When running in regular mode, this rule will only report constants with six
        or more significant figures, as a standard 32-bit floating point can store
        these without issue. However, precision may still be unexpectedly lost in
        operations between 32-bit floats, so strict mode will require _all_ literal
        floating points to use a suffix.

        This rule does not concern constants that make use of exponentiation, such as
        `8.85418782e-12`, which would also lose precision compared to `8.85418782d-12`,
        or better yet `8.85418782e-12_dp`.
        "
    }
}

// Avoid numbered kind suffixes
// -----------------------------

fn avoid_numbered_kind_suffixes(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    // Given a number literal, match anything suffixed with plain number.
    // TODO Match either int or real, change error message accordingly
    let re = Regex::new(r"_\d+$").unwrap();

    let query_txt = "(number_literal) @num";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let txt = capture.node.utf8_text(src.as_bytes());
            match txt {
                Ok(x) => {
                    if re.is_match(x) {
                        let msg = format!(
                            "Instead of number literal suffix in {}, use parameter suffix \
                            from 'iso_fortran_env' or 'selected_int/real_kind'",
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

pub struct AvoidNumberedKindSuffixes {}

impl Rule for AvoidNumberedKindSuffixes {
    fn method(&self) -> Method {
        Method::Tree(Box::new(avoid_numbered_kind_suffixes))
    }

    fn explain(&self) -> &str {
        "
        Using an integer literal as a kind specifier gives no guarantees regarding the
        precision of the type, as kind numbers are not required to match the number of
        bytes. It is recommended to set integer parameters using `selected_int_kind`
        and/or 'selected_real_kind':


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
    fn test_number_literal_kinds() {
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
        test_tree_method(
            avoid_number_literal_kinds,
            source,
            Some(expected_violations),
        );
    }

    #[test]
    fn test_non_standard_byte_specifier() {
        let source = dedent(
            "
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
            ",
        );
        let expected_violations = [(2, 8), (4, 11), (5, 10), (15, 8), (16, 12)]
            .iter()
            .map(|(line, col)| {
                violation!("Avoid non-standard 'type*N', prefer 'type(N)'", *line, *col)
            })
            .collect();
        test_tree_method(
            avoid_non_standard_byte_specifier,
            source,
            Some(expected_violations),
        );
    }

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
        test_tree_method(avoid_double_precision, source, Some(expected_violations));
    }

    #[test]
    fn test_floating_point_without_suffixes() {
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
        let expected = &[(4, 29, "1.234567"), (7, 29, "9.876")];
        for strict in [true, false] {
            let rule = UseFloatingPointSuffixes { strict: strict };
            // If strict, expect both errors. Otherwise, the second isn't sufficiently
            // precise to trigger the rule.
            let errors = if strict {
                &expected[..]
            } else {
                &expected[..1]
            };
            if let Method::Tree(func) = rule.method() {
                let expected_violations = errors
                    .iter()
                    .map(|(line, col, num)| {
                        let msg = format!(
                            "Floating point literal {} specified without a kind suffix",
                            num,
                        );
                        violation!(&msg, *line, *col)
                    })
                    .collect();
                test_tree_method(func, &source, Some(expected_violations));
            } else {
                panic!("Returning wrong kind of method!");
            };
        }
    }

    #[test]
    fn test_avoid_numbered_suffixes() {
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
                    'iso_fortran_env' or 'selected_int/real_kind'",
                    num,
                );
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(
            avoid_numbered_kind_suffixes,
            source,
            Some(expected_violations),
        );
    }
}
