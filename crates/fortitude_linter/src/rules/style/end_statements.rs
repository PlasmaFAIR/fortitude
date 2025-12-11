use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
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
#[derive(ViolationMetadata)]
pub(crate) struct UnnamedEndStatement {
    replacement: String,
}

impl AlwaysFixableViolation for UnnamedEndStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "end statement should be named.".to_string()
    }

    fn fix_title(&self) -> String {
        let Self { replacement } = self;
        format!("Write as '{replacement}'.")
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
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
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

        // Preserve existing case of end statement
        let text = node.to_text(src.source_text())?;
        let is_lower_case = text == text.to_lowercase();
        let end_statement = if is_lower_case {
            format!("end {statement}")
        } else {
            format!("end {statement}").to_uppercase()
        };

        let replacement = format!("{end_statement} {name}");
        let fix = Fix::safe_edit(node.edit_replacement(src, replacement.clone()));
        some_vec![Diagnostic::from_node(Self { replacement }, node).with_fix(fix)]
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
