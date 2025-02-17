use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

// TODO Check that 'used' entity is actually used somewhere

/// ## What it does
/// Checks whether `use` statements are used correctly.
///
/// ## Why is this bad?
/// When using a module, it is recommended to add an 'only' clause to specify which
/// components you intend to use:
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use, intrinsic :: iso_fortran_env
///
/// ! Better
/// use, intrinsic :: iso_fortran_env, only: int32, real64
/// ```
///
/// This makes it easier for programmers to understand where the symbols in your
/// code have come from, and avoids introducing many unneeded components to your
/// local scope.
#[derive(ViolationMetadata)]
pub(crate) struct UseAll {}

impl Violation for UseAll {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'use' statement missing 'only' clause".to_string()
    }
}

impl AstRule for UseAll {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.child_with_name("included_items").is_none() {
            return some_vec![Diagnostic::from_node(UseAll {}, node)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["use_statement"]
    }
}

/// ## What it does
/// Checks whether `use` statements for intrinic modules specify `intrinsic` or
/// `non_intrinsic`.
///
/// ## Why is this bad?
/// The compiler will default to using a non-intrinsic module, if there is one,
/// so not specifying the `intrinsic` modifier on intrinsic modules may lead to
/// the compiler version being shadowed by a different module with the same name.
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use :: iso_fortran_env, only: int32, real64
///
/// ! Better
/// use, intrinsic :: iso_fortran_env, only: int32, real64
/// ```
///
/// This ensures the compiler will use the built-in module instead of a different
/// module with the same name.
#[derive(ViolationMetadata)]
pub(crate) struct MissingIntrinsic {}

const INTRINSIC_MODULES: &[&str] = &[
    "iso_fortran_env",
    "iso_c_binding",
    "ieee_exceptions",
    "ieee_artimetic",
    "ieee_features",
];

impl Violation for MissingIntrinsic {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'use' for intrinsic module missing 'intrinsic' modifier".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'intrinsic'".to_string())
    }
}

impl AstRule for MissingIntrinsic {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let module_name = node
            .child_with_name("module_name")?
            .to_text(_src.source_text())?
            .to_lowercase();

        if INTRINSIC_MODULES.iter().any(|&m| m == module_name)
            && node
                .children(&mut node.walk())
                .filter_map(|child| child.to_text(_src.source_text()))
                .all(|child| child != "intrinsic" && child != "non_intrinsic")
        {
            let intrinsic = if node.child(1)?.kind() == "::" {
                ", intrinsic"
            } else {
                ", intrinsic ::"
            };

            let use_field = node
                .children(&mut node.walk())
                .find(|&child| child.to_text(_src.source_text()) == Some("use"))?;
            let start_pos = TextSize::try_from(use_field.end_byte()).unwrap();
            let fix = Fix::unsafe_edit(Edit::insertion(intrinsic.to_string(), start_pos));

            return some_vec![Diagnostic::from_node(MissingIntrinsic {}, node).with_fix(fix)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["use_statement"]
    }
}
