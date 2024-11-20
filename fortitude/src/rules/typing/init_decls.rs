use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for local variables with implicit `save`
///
/// ## Why is this bad?
/// Initialising procedure local variables in their declaration gives them an
/// implicit `save` attribute: the initialisation is only done on the first call
/// to the procedure, and the variable retains its value on exit.
///
/// ## Examples
/// For example, this subroutine:
///
/// ```f90
/// subroutine example()
///   integer :: var = 1
///   print*, var
///   var = var + 1
/// end subroutine example
/// ```
///
/// when called twice:
///
/// ```f90
/// call example()
/// call example()
/// ```
///
/// prints `1 2`, when it might be expected to print `1 1`.
///
/// Adding the `save` attribute makes it clear that this is the intention:
///
/// ```f90
/// subroutine example()
///   integer, save :: var = 1
///   print*, var
///   var = var + 1
/// end subroutine example
/// ```
///
/// Unfortunately, in Fortran there is no way to disable this behaviour, and so if it
/// is not intended, it's necessary to have a separate assignment statement:
///
/// ```f90
/// subroutine example()
///   integer :: var
///   var = 1
///   print*, var
///   var = var + 1
/// end subroutine example
/// ```
///
/// If the variable's value is intended to be constant, then use the `parameter`
/// attribute instead:
///
/// ```f90
/// subroutine example()
///   integer, parameter :: var = 1
///   print*, var
/// end subroutine example
/// ```
#[violation]
pub struct InitialisationInDeclaration {
    name: String,
}

impl Violation for InitialisationInDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("'{name}' is initialised in its declaration and has no explicit `save` or `parameter` attribute")
    }
}

impl AstRule for InitialisationInDeclaration {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        // Only check in procedures
        node.ancestors().find(|parent| {
            ["function", "subroutine", "module_procedure"].contains(&parent.kind())
        })?;

        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Init in declaration ok for save and parameter
        if declaration
            .children_by_field_name("attribute", &mut declaration.walk())
            .filter_map(|attr| attr.to_text(src))
            .any(|attr_name| ["save", "parameter"].contains(&attr_name.to_lowercase().as_str()))
        {
            return None;
        }

        let name = node.child_by_field_name("left")?.to_text(src)?.to_string();
        some_vec![Diagnostic::from_node(Self { name }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["init_declarator"]
    }
}
