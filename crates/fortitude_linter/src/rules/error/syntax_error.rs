use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;

use ruff_macros::derive_message_formats;
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
#[derive(ViolationMetadata)]
pub(crate) struct SyntaxError {}

impl Violation for SyntaxError {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Syntax error".to_string()
    }
}

impl AstRule for SyntaxError {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        some_vec![context.create_diagnostic(Self {}, node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["ERROR"]
    }
}
