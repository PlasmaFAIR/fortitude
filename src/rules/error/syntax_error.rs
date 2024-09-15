use crate::{Method, Rule, Violation};
use tree_sitter::Node;

/// Rules that check for syntax errors.

fn syntax_error(node: &Node, _src: &str) -> Option<Violation> {
    Some(Violation::from_node("syntax_error", node))
}

pub struct SyntaxError {}

impl Rule for SyntaxError {
    fn method(&self) -> Method {
        Method::Tree(syntax_error)
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

    fn entrypoints(&self) -> Vec<&str> {
        vec!["ERROR"]
    }
}
