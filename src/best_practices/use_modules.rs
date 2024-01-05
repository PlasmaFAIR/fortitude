/// Defines rules that check whether functions and subroutines are defined within modules,
/// submodules, or interfaces. It is also acceptable to define nested functions or subroutines.

// TODO Need to consider modularisation of (abstract) interface blocks.
// TODO Add tests.

use tree_sitter::Node;
use crate::rules;

const CODE: rules::Code = rules::Code::new(rules::Category::BestPractices, 1);
const MSG: &str = "Functions and subroutines should be contained within (sub)modules, program \
    blocks, or interfaces. Fortran compilers are unable to perform type checks and conversions \
    on functions defined outside of these scopes, and this is a common source of bugs.";

/// Checks if a function (or subroutine) is defined within a module, submodule, interface,
/// function, or subroutine. Free-standing functions return a Violation. Well-contained functions
/// return None. Assumes that the input node is a function or subroutine.
fn use_modules_violation(node: &Node) -> Option<rules::Violation> {
    let parent = node.parent()?;
    match parent.kind() {
        "translation_unit" => Some(
            rules::Violation::from_node(
                node,
                CODE,
                "Function or subroutine not contained within (sub)module, program, or interface",
            )
        ),
        _ => None,
    }
}

/// Checks if given node is a function or subroutine. If so, checks if it's contained within an
/// appropriate scope. If the node isn't a function, the child nodes are checked one by one. If the
/// node isn't in an appropriate scope, adds a a violation to the violations vector.
fn use_modules_method(node: &Node) -> Vec<rules::Violation> {
    let kind = node.kind();
    match kind {
        "function" | "subroutine" => {
            match use_modules_violation(node) {
                Some(x) => vec![x],
                _ => vec![],
            }
        },
        _ => {
            let mut violations: Vec<rules::Violation> = Vec::new();
            for child in node.children(&mut node.walk()){
                violations.append(&mut use_modules_method(&child));
            }
            violations
        },
    }
}

pub fn use_modules() -> rules::Rule {
    rules::Rule::new(CODE, rules::Method::Tree(use_modules_method), MSG)
}
