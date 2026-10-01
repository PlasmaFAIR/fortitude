use crate::diagnostics::{Diagnostic, Fix, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use crate::rules::utilities;
use fortitude_macros::{ViolationMetadata};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

fn map_gfortran_extensions_to_instrinsics(name: &str) -> Option<&'static str> {
    match name {
        "SRAND" => Some("RANDOM_SEED"),
        "RAND" => Some("RANDOM_NUMBER"),
        _ => None,
    }
}

/// ## What does it do?
/// Checks for gfortran random number extensions, srand and rand, and suggests
/// to replace them with fortran standard intrinsics random_seed and random_number
/// respectively.
///
/// ## Why is this bad?
#[derive(ViolationMetadata)]
pub(crate) struct GFortranRandomExtension {
    func: String,
    new_func: String,
}

impl Violation for GFortranRandomExtension {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { func, .. } = self;
        format!("gfortran extension function'{func}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { new_func, .. } = self;
        Some(format!("Use standard intrinsic '{new_func}'"))
    }
}

impl AstRule for GFortranRandomExtension {
    fn check<'a>(context: &'a CheckContext, node: &'a Node) -> Option<Vec<Diagnostic>> {
        let name_node = node.child_with_name("identifier")?;
        let func = name_node.text();

        let new_func = map_gfortran_extensions_to_instrinsics(func.to_uppercase().as_str())?;
        let matched_case = utilities::match_original_case(func, new_func)?;

        let fix = Fix::unsafe_edit(name_node.edit_replacement(matched_case.clone()));

        some_vec![
            context
                .create_diagnostic(
                    Self {
                        func: func.to_string(),
                        new_func: matched_case
                    },
                    name_node
                )
                .with_fix(fix)
        ]
    }
    
    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression"]
    }
}
