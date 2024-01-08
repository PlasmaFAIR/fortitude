use crate::rules;
/// Defines rules that check whether functions and subroutines are defined within modules,
/// submodules, or interfaces. It is also acceptable to define nested functions or subroutines.
// TODO Need to consider modularisation of (abstract) interface blocks.
// TODO Add tests.
use tree_sitter::Node;

const CODE: rules::Code = rules::Code::new(rules::Category::BestPractices, 1);
const MSG: &str = "Functions and subroutines should be contained within (sub)modules, program \
    blocks, or interfaces. Fortran compilers are unable to perform type checks and conversions \
    on functions defined outside of these scopes, and this is a common source of bugs.";

/// Checks if a function (or subroutine) is defined within a module, submodule, interface,
/// function, or subroutine. Free-standing functions return a Violation. Well-contained functions
/// return None. Assumes that the input node is a function or subroutine.
fn use_modules_violation(kind: &str, node: &Node) -> Option<rules::Violation> {
    let parent = node.parent()?;
    match parent.kind() {
        "translation_unit" => Some(rules::Violation::from_node(
            node,
            CODE,
            format!(
                "{} not contained within (sub)module, program, or interface",
                kind
            )
            .as_str(),
        )),
        _ => None,
    }
}

/// Checks if given node is a function or subroutine. If so, checks if it's contained within an
/// appropriate scope. If the node isn't a function, the child nodes are checked one by one. If the
/// node isn't in an appropriate scope, adds a a violation to the violations vector.
fn use_modules_method(node: &Node) -> Vec<rules::Violation> {
    let kind = node.kind();
    match kind {
        "function" | "subroutine" => match use_modules_violation(kind, node) {
            Some(x) => vec![x],
            _ => vec![],
        },
        _ => {
            let mut violations: Vec<rules::Violation> = Vec::new();
            for child in node.children(&mut node.walk()) {
                violations.extend(use_modules_method(&child));
            }
            violations
        }
    }
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
        let mut violations = Vec::new();
        match rule.method() {
            rules::Method::Tree(f) => {
                violations.extend(f(&root));
            }
            _ => {
                panic!();
            }
        }
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
