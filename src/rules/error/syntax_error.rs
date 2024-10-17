use crate::settings::Settings;
use crate::{some_vec, ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Rules that check for syntax errors.

pub struct SyntaxError {}

impl Rule for SyntaxError {
    fn new(_settings: &Settings) -> Self {
        SyntaxError {}
    }

    fn explain(&self) -> &'static str {
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

impl ASTRule for SyntaxError {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Violation>> {
        some_vec![Violation::from_node("syntax_error", node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["ERROR"]
    }
}
