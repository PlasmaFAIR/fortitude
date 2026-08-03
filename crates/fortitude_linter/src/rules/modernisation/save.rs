use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

use crate::{AstRule, CheckContext, kind_ids, settings::FortranStandard};
use fortitude_sitter::traits::TextRanged;

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

impl AlwaysFixableViolation for SuperfluousSave {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("save {} is superfluous at the module level", self.entity)
    }

    fn fix_title(&self) -> String {
        "Remove `save`".to_string()
    }
}

impl AstRule for SuperfluousSave {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // Only F2008 and later made `save` at the module level implicit
        if context.settings().target_std < FortranStandard::F2008 {
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
                .find(|c| c.text() == "save")?;

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
                context
                    .create_diagnostic(
                        Self {
                            entity: "attribute"
                        },
                        save_qualifier
                    )
                    .with_fix(Fix::safe_edit(Edit::deletion(
                        start_node.start_textsize(),
                        save_qualifier.end_textsize()
                    )))
            ]
        } else {
            let save_statement = node.child_with_name("save_statement")?;

            some_vec![
                context
                    .create_diagnostic(
                        Self {
                            entity: "statement"
                        },
                        save_statement
                    )
                    .with_fix(Fix::safe_edit(save_statement.edit_delete()))
            ]
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["module", "submodule", "variable_declaration"]
    }
}
