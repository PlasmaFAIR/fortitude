use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};

use crate::{
    AstRule, FromAstNode, ast::FortitudeNode, settings::FortranStandard, traits::TextRanged,
};

/// ## What it does
/// Checks for unnecessary `save` statements and qualifiers
///
/// ## Why is this bad?
/// Since Fortran 2008, module variables are implicitly saved. Save statements
/// and attributes can safely be removed.
///
/// ## Example
/// ```f90
/// module example
///     integer, save :: a
/// end module example
/// ```
/// or
/// ```f90
/// module example
///     save
///
///     integer :: a
/// end module example
/// ```
///
/// Use instead:
/// ```f90
/// module example
///     integer :: a
/// end module example
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousSave {
    entity: &'static str,
}

impl Violation for SuperfluousSave {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("save {} is superfluous at the module level", self.entity)
    }

    fn fix_title(&self) -> Option<String> {
        Some("Remove `save`".to_string())
    }
}

impl AstRule for SuperfluousSave {
    fn check(
        settings: &crate::settings::CheckSettings,
        node: &tree_sitter::Node,
        source: &ruff_source_file::SourceFile,
        _symbol_table: &crate::ast::symbol_table::SymbolTables,
    ) -> Option<Vec<ruff_diagnostics::Diagnostic>> {
        // Only F2008 and later made `save` at the module level implicit
        if settings.target_std < FortranStandard::F2008 {
            return None;
        }

        if node.kind() == "variable_declaration" {
            if node
                .parent()
                .is_none_or(|x| !matches!(x.kind(), "module" | "submodule"))
            {
                return None;
            }

            let save_qualifier = node
                .named_children(&mut node.walk())
                .filter(|c| c.grammar_name() == "type_qualifier")
                .find(|c| c.to_text(source.source_text()) == Some("save"))?;

            let start_node = match save_qualifier.prev_sibling() {
                None => save_qualifier,
                Some(prev) => {
                    if prev.grammar_name() == "," {
                        prev
                    } else {
                        save_qualifier
                    }
                }
            };

            some_vec![
                Diagnostic::from_node(
                    Self {
                        entity: "attribute"
                    },
                    &save_qualifier
                )
                .with_fix(Fix::unsafe_edit(Edit::deletion(
                    start_node.start_textsize(),
                    save_qualifier.end_textsize()
                )))
            ]
        } else {
            let save_statement = node.child_with_name("save_statement")?;

            some_vec![
                Diagnostic::from_node(
                    Self {
                        entity: "statement"
                    },
                    &save_statement
                )
                .with_fix(Fix::unsafe_edit(Edit::deletion(
                    save_statement.start_textsize(),
                    save_statement.end_textsize()
                )))
            ]
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module", "submodule", "variable_declaration"]
    }
}
