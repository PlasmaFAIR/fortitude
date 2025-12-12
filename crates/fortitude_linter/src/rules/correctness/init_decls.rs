use crate::ast::FortitudeNode;
use crate::ast::types::{AttributeKind, get_name_node_of_declarator};
use crate::settings::CheckSettings;
use crate::{AstRule, FromAstNode, SymbolTables};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
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
#[derive(ViolationMetadata)]
pub(crate) struct InitialisationInDeclaration {
    name: String,
}

impl Violation for InitialisationInDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!(
            "'{name}' is initialised in its declaration and has no explicit `save` or `parameter` attribute"
        )
    }
}

impl AstRule for InitialisationInDeclaration {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        // Only check in procedures
        node.ancestors().find(|parent| {
            ["function", "subroutine", "module_procedure"].contains(&parent.kind())
        })?;

        let name = get_name_node_of_declarator(node).to_text(src)?.to_string();
        let decl = symbol_table.get(name.as_str())?;
        if decl.has_any_attributes(&[AttributeKind::Save, AttributeKind::Parameter]) {
            return None;
        }

        some_vec![Diagnostic::from_node(Self { name }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["init_declarator"]
    }
}

/// ## What it does
/// Checks for local pointer variables with implicit `save`
///
/// ## Why is this bad?
/// Initialising procedure local pointer variables in their declaration gives them
/// an implicit `save` attribute: the initialisation is only done on the first call
/// to the procedure, and the pointer retains its associated status on exit.
///
/// However, this associated status makes no guarantee that the target of the pointer
/// is valid or stays allocated between procedure calls - potentially leading to cases
/// where future calls into a procedure will reference an unallocated variable.
///
/// ## Examples
/// For example, this subroutine:
///
/// ```f90
/// subroutine example()
///   integer, target  :: bad
///   integer, pointer :: var => null()
///
///   if (.not.associated(var)) then
///     bad = 1
///     var => bad
///   end if
///
///   print *, var
/// end subroutine example
/// ```
///
/// when called twice
///
/// ```f90
/// call example()
/// call doAnotherBigThing()
/// call example()
/// ```
///
/// will implicitly save the association of `var` with the variable `bad` in the first
/// procedure call, but that variable most likely won't be at the same location in memory in
/// the next call, so the value printed in the second procedure call might not be `1`.
///
/// Adding the `save` attribute makes it clear that this is the intention, however you should
/// also ensure that the target variable is also saved.
///
/// ```f90
/// subroutine example()
///   integer, target,  save :: bad
///   integer, pointer, save :: var => null()
///
///   if (.not.associated(var)) then
///     bad = 1
///     var => bad
///   end if
///
///   print *, var
/// end subroutine example
/// ```
///
/// Unfortunately, in Fortran there is no way to disable this behaviour, and so if it
/// is not intended, it's necessary to have a separate nullification statement before use:
///
/// ```f90
/// subroutine example()
///   integer, target  :: bad
///   integer, pointer :: var
///
///   var => null()  ! or use nullify(var)
///
///   if (.not.associated(var)) then
///     bad = 1
///     var => bad
///   end if
///
///   print *, var
/// end subroutine example
/// ```
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Section 8.5.3/8.5.4.
/// - Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
///   University Press, Rule 74, p. 99.
#[derive(ViolationMetadata)]
pub(crate) struct PointerInitialisationInDeclaration {
    name: String,
}

impl Violation for PointerInitialisationInDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!(
            "Pointer '{name}' is initialized in its declaration and has no explicit `save` attribute"
        )
    }
}

impl AstRule for PointerInitialisationInDeclaration {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        // Only check in procedures
        node.ancestors().find(|parent| {
            ["function", "subroutine", "module_procedure"].contains(&parent.kind())
        })?;

        let var = get_name_node_of_declarator(node);
        let name = var.to_text(src)?.to_string();
        let decl = symbol_table.get(name.as_str())?;
        if decl.has_attribute(AttributeKind::Save) {
            return None;
        }

        // Array syntax on the variable name
        if let Some(arr) = var.child_with_name("identifier") {
            let name = arr.to_text(src)?.to_string();
            return some_vec![Diagnostic::from_node(Self { name }, node)];
        }

        some_vec![Diagnostic::from_node(Self { name }, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["pointer_init_declarator"]
    }
}
