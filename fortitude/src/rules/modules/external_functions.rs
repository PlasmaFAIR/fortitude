use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for any functions and subroutines not defined within modules (or one
/// of a few acceptable alternatives).
///
/// ## Why is this bad?
/// Functions and subroutines should be contained within (sub)modules or programs.
/// Fortran compilers are unable to perform type checks and conversions on functions
/// defined outside of these scopes, and this is a common source of bugs.
#[violation]
pub struct ExternalFunction {
    procedure: String,
}

impl Violation for ExternalFunction {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ExternalFunction { procedure } = self;
        format!("{procedure} not contained within (sub)module or program")
    }
}

impl AstRule for ExternalFunction {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.parent()?.kind() == "translation_unit" {
            let procedure_stmt = node.child(0)?;
            let procedure = node.kind().to_string();
            return some_vec![Diagnostic::from_node(
                ExternalFunction { procedure },
                &procedure_stmt
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}
