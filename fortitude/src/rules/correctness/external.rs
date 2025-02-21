use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
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
#[derive(ViolationMetadata)]
pub(crate) struct ExternalProcedure {
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

/// ## What it does
/// Checks for any functions and subroutines not defined within modules (or one
/// of a few acceptable alternatives).
///
/// ## Why is this bad?
/// Functions and subroutines should be contained within (sub)modules or programs.
/// Fortran compilers are unable to perform type checks and conversions on functions
/// defined outside of these scopes, and this is a common source of bugs.
#[derive(ViolationMetadata)]
pub(crate) struct ProcedureNotInModule {
    procedure: String,
}

impl Violation for ProcedureNotInModule {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ProcedureNotInModule { procedure } = self;
        format!("{procedure} not contained within (sub)module or program")
    }
}

impl AstRule for ProcedureNotInModule {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.parent()?.kind() == "translation_unit" {
            let procedure_stmt = node.child(0)?;
            let procedure = node.kind().to_string();
            return some_vec![Diagnostic::from_node(
                ProcedureNotInModule { procedure },
                &procedure_stmt
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}
