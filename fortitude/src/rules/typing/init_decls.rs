use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, FromASTNode};
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
/// ```fortran
/// subroutine example()
///   integer :: var = 1
///   print*, var
///   var = var + 1
/// end subroutine example
/// ```
///
/// when called twice:
///
/// ```fortran
/// call example()
/// call example()
/// ```
///
/// prints `1 2`, when it might be expected to print `1 1`.
///
/// Adding the `save` attribute makes it clear that this is the intention:
///
/// ```fortran
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
/// ```fortran
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
/// ```fortran
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

impl ASTRule for InitialisationInDeclaration {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_init_decl() -> anyhow::Result<()> {
        let source = test_file(
            "
            module test
              integer :: global = 0  ! Ok at module level

              type :: my_type
                integer :: component = 1  ! Ok in types
              end type
            contains

            subroutine init_decl1()
              integer :: foo = 1
            end subroutine init_decl1

            subroutine init_decl2()
              integer, save :: foo = 1  ! Ok with explicit save
              integer, parameter :: bar = 2  ! Ok as parameter
            end subroutine init_decl2

            subroutine init_decl3()
              integer :: foo, bar = 1, quazz, zapp = 2
            end subroutine no_init_decl3

            end module test
            ",
        );
        let expected: Vec<_> = [
            (10, 13, 10, 20, "foo"),
            (19, 18, 19, 25, "bar"),
            (19, 34, 19, 42, "zapp"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, variable)| {
            Diagnostic::from_start_end_line_col(
                InitialisationInDeclaration {
                    name: variable.to_string(),
                },
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let actual = InitialisationInDeclaration::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
