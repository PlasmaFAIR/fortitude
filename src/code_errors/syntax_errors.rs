use crate::core::{Method, Rule, Violation};
use tree_sitter::Node;

/// Rules that check for syntax errors.

fn find_syntax_errors(node: &Node) -> Vec<Violation> {
    // TODO There should be a way to achieve this just using iterators, without
    //      returning intermediates.
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
        if child.is_error() {
            violations.push(Violation::from_node("syntax error", &child));
        }
        violations.extend(find_syntax_errors(&child));
    }
    violations
}

pub struct SyntaxErrors {}

impl Rule for SyntaxErrors {
    fn method(&self) -> Method {
        Method::Tree(Box::new(|node, _| find_syntax_errors(node)))
    }

    fn explain(&self) -> &str {
        "
        This rule reports any syntax errors reported by Fortitude's Fortran parser.
        This may indicate an error with your code, an aspect of Fortran not recognised
        by the parser, or a non-standard extension to Fortran that our parser can't
        handle, such as a pre-processor.

        If this rule is reporting valid Fortran, please let us know, as it's likely a
        bug in our code or in our parser!
        "
    }
}
