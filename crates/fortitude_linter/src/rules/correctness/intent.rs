use crate::ast::types::AttributeKind;
use crate::ast::{FortitudeNode, types::HasName};
use crate::diagnostics::{Diagnostic, Violation};
use crate::settings::FortranStandard;
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, field};
use itertools::Itertools;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for missing `intent` on dummy arguments
///
/// ## Why is this bad?
/// Procedure dummy arguments should have an explicit `intent`
/// attributes. This can help catch logic errors, potentially improve
/// performance, as well as serving as documentation for users of
/// the procedure.
///
/// Arguments with `intent(in)` are read-only input variables, and cannot be
/// modified by the routine.
///
/// Arguments with `intent(out)` are output variables, and their value
/// on entry into the routine can be safely ignored -- technically,
/// they become undefined on entry, which includes deallocation and/or
/// finalisation.
///
/// Finally, `intent(inout)` arguments can be both read and modified
/// by the routine. If an `intent` is not specified, it will default
/// to essentially `intent(inout)` -- however, this can be dangerous
/// if passing literals or expressions that can't be modified.
///
/// This rule will permit the absence of `intent` for dummy arguments
/// that include the `value` attribute. It will also permit `pointer`
/// dummy arguments that lack an `intent` attribute in standards prior
/// to Fortran 2003, in which `pointer` dummy arguments were not
/// allowed to have `intent`.
#[derive(ViolationMetadata)]
pub(crate) struct MissingIntent {
    entity: String,
    name: String,
}

impl Violation for MissingIntent {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity, name } = self;
        format!("{entity} argument '{name}' missing 'intent' attribute")
    }
}

impl AstRule for MissingIntent {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let entity = node.parent()?.kind().to_string();
        Some(
            node.child_by_field_id(field!("parameters").into())?
                .named_children(&mut node.walk())
                .filter_map(|param| {
                    // Get variable declaration
                    context
                        .symbol_table()
                        .get_var(param.to_text(context.source_text())?)
                })
                .filter(|param| {
                    // Procedures are not allowed intent
                    !(param.type_().is_procedure() || param.has_attribute(AttributeKind::External))
                })
                .filter(|param| {
                    // Intent only allowed on pointers after F2003
                    !(context.settings().target_std < FortranStandard::F2003
                        && param.has_attribute(AttributeKind::Pointer))
                })
                .filter(|param| {
                    // Already has intent!
                    !param
                        .attributes()
                        .iter()
                        .any(|attr| attr.kind().is_intent() || attr.kind().is_value())
                })
                .map(|param| {
                    context.create_diagnostic(
                        Self {
                            entity: entity.clone(),
                            name: param.name().to_string(),
                        },
                        param.textrange(),
                    )
                })
                .collect_vec(),
        )
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["function_statement", "subroutine_statement"]
    }
}
