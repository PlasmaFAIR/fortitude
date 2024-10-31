use crate::settings::Settings;
use crate::{some_vec, ASTRule, FortitudeViolation, Rule};

use ruff_diagnostics::Violation;
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for syntax errors
///
/// This rule reports any syntax errors reported by Fortitude's Fortran parser.
/// This may indicate an error with your code, an aspect of Fortran not recognised
/// by the parser, or a non-standard extension to Fortran that our parser can't
/// handle, such as a pre-processor.
///
/// If this rule is reporting valid Fortran, please let us know, as it's likely a
/// bug in our code or in our parser!
#[violation]
pub struct SyntaxError {}

impl Violation for SyntaxError {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Syntax error")
    }
}

impl Rule for SyntaxError {
    fn new(_settings: &Settings) -> Self {
        SyntaxError {}
    }
}

impl ASTRule for SyntaxError {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<FortitudeViolation>> {
        some_vec![FortitudeViolation::from_node("syntax_error", node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["ERROR"]
    }
}
