/// Defines rules that raise errors if implicit typing is in use.

// TODO require implicit none in interface functions (code 11)
// TODO report use of function `implicit none` when its set on the enclosing module (code 12)

use tree_sitter::Node;
use crate::rules;

const CODE10: rules::Code = rules::Code::new(rules::Category::BestPractices, 10);
const MSG10: &str = "'implicit none' should be used in all modules and programs, as implicit
    typing reduces the readability of code and increases the chances of typing errors.";

fn use_implicit_none_violation(kind: &str, node: &Node) -> Option<rules::Violation> {
    for child in node.children(&mut node.walk()){
        if child.kind() == "implicit_statement" {
            match child.child(1) {
                Some(x) => {
                    if x.kind() == "none" {
                        return None;
                    }
                },
                None => {
                    continue;
                }
            }
        }
    }
    Some(
        rules::Violation::from_node(
            node, 
            CODE10,
            format!("{} missing 'implicit none'", kind).as_str(),
        )
    )
}

fn use_implicit_none_method(node: &Node) -> Vec<rules::Violation> {
    let kind = node.kind();
    match kind {
        "module" | "submodule" | "program" => {
            match use_implicit_none_violation(kind, node) {
                Some(x) => vec![x],
                _ => vec![],
            }
        },
        _ => {
            let mut violations: Vec<rules::Violation> = Vec::new();
            for child in node.children(&mut node.walk()){
                violations.extend(use_implicit_none_method(&child));
            }
            violations
        },
    }
}

pub fn use_implicit_none() -> rules::Rule {
    rules::Rule::new(CODE10, rules::Method::Tree(use_implicit_none_method), MSG10)
}
