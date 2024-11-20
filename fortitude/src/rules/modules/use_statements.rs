use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
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
#[violation]
pub struct UseAll {}

impl Violation for UseAll {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("'use' statement missing 'only' clause")
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
