use crate::{Method, Rule, Violation};
use tree_sitter::{Node, Query};

/// Defines rules that check whether functions and subroutines are defined within modules (or one
/// of a few acceptable alternatives).

fn external_fucntion(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let query_txt = "(translation_unit [(function) @func (subroutine) @sub])";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let node = capture.node;
            let msg = format!(
                "{} not contained within (sub)module or program",
                node.kind()
            );
            violations.push(Violation::from_node(&msg, &node));
        }
    }
    violations
}

pub struct ExternalFunction {}

impl Rule for ExternalFunction {
    fn method(&self) -> Method {
        Method::Tree(external_fucntion)
    }

    fn explain(&self) -> &str {
        "
        Functions and subroutines should be contained within (sub)modules or programs.
        Fortran compilers are unable to perform type checks and conversions on functions
        defined outside of these scopes, and this is a common source of bugs.
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
    fn test_function_not_in_module() {
        let source = dedent(
            "
            integer function double(x)
              integer, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              integer, intent(inout) :: x
              x = 3 * x
            end subroutine
            ",
        );
        let expected_violations = [(2, 1, "function"), (7, 1, "subroutine")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("{} not contained within (sub)module or program", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(external_fucntion, source, Some(expected_violations));
    }

    #[test]
    fn test_function_in_module() {
        let source = "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function

                subroutine triple(x)
                  integer, intent(inout) :: x
                  x = 3 * x
                end subroutine
            end module
            ";
        test_tree_method(external_fucntion, source, None);
    }
}
