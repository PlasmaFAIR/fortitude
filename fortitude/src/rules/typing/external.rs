use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

use crate::{ast::FortitudeNode, settings::Settings, AstRule, FromAstNode};

/// ## What does it do?
/// Checks for procedures declared with just `external`
///
/// ## Why is this bad?
/// Compilers are unable to check external procedures without an explicit
/// interface for errors such as wrong number or type of arguments.
///
/// If the procedure is in your project, put it in a module (see
/// `external-function`), or write an explicit interface.
#[violation]
pub struct ExternalProcedure {
    name: String,
}

impl Violation for ExternalProcedure {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("'{name}' declared as `external`")
    }

    fn fix_title(&self) -> Option<String> {
        Some(format!("Write an explicit interface"))
    }
}

impl AstRule for ExternalProcedure {
    fn check(_settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node
            .child_with_name("type_qualifier")?
            .to_text(source.source_text())?
            .to_lowercase()
            != "external"
        {
            return None;
        }

        let name = node
            .child_by_field_name("declarator")?
            .to_text(source.source_text())?
            .to_string();
        some_vec!(Diagnostic::from_node(Self { name }, node))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["variable_modification"]
    }
}
