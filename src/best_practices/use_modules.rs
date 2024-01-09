use crate::parser::fortran_language;
use crate::rules;
/// Defines rules that check whether functions and subroutines are defined within modules,
/// submodules, or interfaces. It is also acceptable to define nested functions or subroutines.
use tree_sitter::{Node, Query};

const CODE: rules::Code = rules::Code::new(rules::Category::BestPractices, 1);
const MSG: &str = "Functions and subroutines should be contained within (sub)modules, program \
    blocks, or interfaces. Fortran compilers are unable to perform type checks and conversions \
    on functions defined outside of these scopes, and this is a common source of bugs.";

fn use_modules_method(root: &Node, src: &str) -> Vec<rules::Violation> {
    let mut violations = Vec::new();
    let query_txt = "(translation_unit [(function) @func (subroutine) @sub])";
    let query = Query::new(fortran_language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for captures in cursor
        .matches(&query, *root, src.as_bytes())
        .map(|x| x.captures)
    {
        for capture in captures {
            let node = capture.node;
            violations.push(rules::Violation::from_node(
                &node,
                CODE,
                format!(
                    "{} not contained within (sub)module, program, or interface",
                    node.kind(),
                )
                .as_str(),
            ));
        }
    }
    violations
}

pub fn use_modules() -> rules::Rule {
    rules::Rule::new(
        CODE,
        rules::Method::Tree(use_modules_method),
        MSG,
        rules::Status::Standard,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::fortran_parser;

    fn test_helper(code: &str, err: Option<Vec<String>>) {
        let mut parser = fortran_parser();
        let tree = parser.parse(&code, None).unwrap();
        let root = tree.root_node();
        let rule = use_modules();
        let violations = use_modules_method(&root, code);
        match err {
            Some(x) => {
                assert_eq!(violations.len(), x.len());
                for (actual, expected) in violations.iter().zip(x) {
                    assert_eq!(actual.to_string(), expected);
                }
            }
            None => {
                // Do nothing!
            }
        }
    }

    #[test]
    fn test_function_not_in_module() {
        let code = "
            integer function double(x)
              integer, intent(in) :: x
              double = 2 * x
            end function
            
            subroutine triple(x)
              integer, intent(inout) :: x
              x = 3 * x
            end subroutine
            ";
        let errs = [2, 7]
            .iter()
            .zip(["function", "subroutine"])
            .map(|(line, msg)| {
                format!(
                    "Line {}: B001 {} not contained within (sub)module, \
                    program, or interface",
                    line, msg,
                )
                .to_string()
            })
            .collect();
        test_helper(code, Some(errs));
    }

    #[test]
    fn test_function_in_module() {
        let code = "
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
        test_helper(code, None);
    }
}
