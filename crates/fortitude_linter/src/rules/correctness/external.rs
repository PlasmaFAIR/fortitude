use crate::diagnostics::{Diagnostic, Violation};
use fortitude_macros::{ViolationMetadata, field, kind, kw};
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

use crate::{AstRule, CheckContext, ast::FortitudeNode, kind_ids};

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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // Exit early if not an external procedure declaration
        node.named_child_with_kind_id(kind!("type_qualifier"))?
            .child_with_kind_id(kw!("external"))?;

        let name = node
            .child_by_field_id(field!("declarator").into())?
            .to_text(context.source_text())?
            .to_string();
        some_vec!(context.create_diagnostic(Self { name }, node))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["variable_modification"]
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.parent()?.kind_id() == kind!("translation_unit") {
            let procedure_stmt = node.child(0)?;
            let procedure = node.kind().to_string();
            return some_vec![
                context.create_diagnostic(ProcedureNotInModule { procedure }, procedure_stmt)
            ];
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["function", "subroutine"]
    }
}
