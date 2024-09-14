use crate::{Method, Rule, Violation};
use tree_sitter::Node;

/// Defines rules that check whether functions and subroutines are defined within modules (or one
/// of a few acceptable alternatives).

fn function_is_at_top_level(node: &Node, _src: &str) -> Option<Violation> {
    if node.parent()?.kind() == "translation_unit" {
        let msg = format!(
            "{} not contained within (sub)module or program",
            node.kind()
        );
        return Some(Violation::from_node(msg, node));
    }
    None
}

fn external_function(node: &Node, _src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "function" || kind == "subroutine" {
            if let Some(x) = function_is_at_top_level(&child, _src) {
                violations.push(x);
            }
        }
        violations.extend(external_function(&child, _src));
    }
    violations
}

pub struct ExternalFunction {}

impl Rule for ExternalFunction {
    fn method(&self) -> Method {
        Method::Tree(external_function)
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
        test_tree_method(external_function, source, Some(expected_violations));
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
        test_tree_method(external_function, source, None);
    }
}
