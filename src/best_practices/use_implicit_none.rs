use crate::rules;
/// Defines rules that raise errors if implicit typing is in use.
// TODO require implicit none in interface functions (code 11)
// TODO report use of function `implicit none` when its set on the enclosing module (code 12)
use tree_sitter::Node;

const CODE10: rules::Code = rules::Code::new(rules::Category::BestPractices, 10);
const MSG10: &str = "'implicit none' should be used in all modules and programs, as implicit
    typing reduces the readability of code and increases the chances of typing errors.";

fn use_implicit_none_violation(kind: &str, node: &Node) -> Option<rules::Violation> {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "implicit_statement" {
            match child.child(1) {
                Some(x) => {
                    if x.kind() == "none" {
                        return None;
                    }
                }
                None => {
                    continue;
                }
            }
        }
    }
    Some(rules::Violation::from_node(
        node,
        CODE10,
        format!("{} missing 'implicit none'", kind).as_str(),
    ))
}

fn use_implicit_none_method(node: &Node) -> Vec<rules::Violation> {
    let kind = node.kind();
    match kind {
        "module" | "submodule" | "program" => match use_implicit_none_violation(kind, node) {
            Some(x) => vec![x],
            _ => vec![],
        },
        _ => {
            let mut violations: Vec<rules::Violation> = Vec::new();
            for child in node.children(&mut node.walk()) {
                violations.extend(use_implicit_none_method(&child));
            }
            violations
        }
    }
}

pub fn use_implicit_none() -> rules::Rule {
    rules::Rule::new(
        CODE10,
        rules::Method::Tree(use_implicit_none_method),
        MSG10,
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
        let rule = use_implicit_none();
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
    fn test_missing_implicit_none() {
        let code = "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ";
        let errs = [2, 6]
            .iter()
            .zip(["module", "program"])
            .map(|(line, msg)| {
                format!("Line {}: B010 {} missing 'implicit none'", line, msg,).to_string()
            })
            .collect();
        test_helper(code, Some(errs));
    }

    #[test]
    fn test_uses_implicit_none() {
        let code = "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function
            end module

            program my_program
                implicit none
                integer, paramter :: x = 2
                write(*,*) x
            end program
            ";
        test_helper(code, None);
    }
}
