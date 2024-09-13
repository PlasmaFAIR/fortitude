use crate::{Method, Rule, Violation};
use lazy_regex::regex_is_match;
use tree_sitter::{Node, Query};
/// Defines rules that discourage the use of the non-standard kind specifiers such as
/// `int*4` or `real*8`. Also prefers the use of `character(len=*)` to
/// `character*(*)`, as although the latter is permitted by the standard, the former is
/// more explicit.

// TODO Add character* rule

fn star_kind(root: &Node, src: &str) -> Vec<Violation> {
    // Note: This does not match 'character*(*)', which should be handled by a different
    // rule.
    let mut violations = Vec::new();
    for query_type in ["function_statement", "variable_declaration"] {
        let query_txt = format!("({} (intrinsic_type) (size) @size)", query_type);
        let query = Query::new(&tree_sitter_fortran::language(), &query_txt).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                match capture.node.utf8_text(src.as_bytes()) {
                    Ok(x) => {
                        // Match anything beginning with a '*' followed by any amount of whitespace
                        // or '&' symbols (in case you like to split your type specifiers over
                        // multiple lines), followed by at least one digit.
                        if regex_is_match!(r"^\*[\s&]*\d+", x) {
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

pub struct StarKind {}

impl Rule for StarKind {
    fn method(&self) -> Method {
        Method::Tree(star_kind)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::test_tree_method;
    use crate::violation;
    use textwrap::dedent;

    #[test]
    fn test_star_kind() {
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
        test_tree_method(star_kind, source, Some(expected_violations));
    }
}
