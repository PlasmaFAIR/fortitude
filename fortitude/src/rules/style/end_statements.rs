use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks that `end` statements include the type of construct they're ending
///
/// ## Why is this bad?
/// End statements should specify what kind of construct they're ending, and the
/// name of that construct. For example, prefer this:
///
/// ```f90
/// module mymodule
///   ...
/// end module mymodule
/// ```
///
/// To this:
///
/// ```f90
/// module mymodule
///   ...
/// end
/// ```
///
/// Or this:
///
/// ```f90
/// module mymodule
///   ...
/// end module
/// ```
///
/// Similar rules apply for many other Fortran statements
#[violation]
pub struct UnnamedEndStatement {
    statement: String,
    name: String,
}

impl Violation for UnnamedEndStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        let UnnamedEndStatement { statement, name } = self;
        format!("end statement should read 'end {statement} {name}'")
    }
}

/// Maps declaration kinds to its name and the kind of the declaration statement node
fn map_declaration(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "module" => ("module", "module_statement"),
        "submodule" => ("submodule", "submodule_statement"),
        "program" => ("program", "program_statement"),
        "function" => ("function", "function_statement"),
        "subroutine" => ("subroutine", "subroutine_statement"),
        "module_procedure" => ("procedure", "module_procedure_statement"),
        "derived_type_definition" => ("type", "derived_type_statement"),
        _ => unreachable!("Invalid entrypoint for AbbreviatedEndStatement"),
    }
}

impl AstRule for UnnamedEndStatement {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        // TODO Also check for optionally labelled constructs like 'do' or 'select'

        // If end node is named, move on.
        // Not catching incorrect end statement name here, as the compiler should
        // do that for us.
        if node.child_with_name("name").is_some() {
            return None;
        }

        let declaration = node.parent()?;
        let (statement, statement_kind) = map_declaration(declaration.kind());
        let statement_node = declaration.child_with_name(statement_kind)?;
        let name_kind = match statement_kind {
            "derived_type_statement" => "type_name",
            _ => "name",
        };
        let name = statement_node
            .child_with_name(name_kind)?
            .to_text(src.source_text())?
            .to_string();
        let statement = statement.to_string();
        some_vec![Diagnostic::from_node(Self { statement, name }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![
            "end_module_statement",
            "end_submodule_statement",
            "end_program_statement",
            "end_function_statement",
            "end_subroutine_statement",
            "end_module_procedure_statement",
            "end_type_statement",
        ]
    }
}
