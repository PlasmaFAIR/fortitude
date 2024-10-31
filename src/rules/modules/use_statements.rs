use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, FortitudeViolation, Rule};
use ruff_diagnostics::Violation;
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
/// ```fortran
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

impl Rule for UseAll {
    fn new(_settings: &Settings) -> Self {
        UseAll {}
    }
}

impl ASTRule for UseAll {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<FortitudeViolation>> {
        if node.child_with_name("included_items").is_none() {
            return some_vec![FortitudeViolation::from_node(UseAll {}, node)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["use_statement"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_use_all() -> anyhow::Result<()> {
        let source = test_file(
            "
            module my_module
                use, intrinsic :: iso_fortran_env, only: real32
                use, intrinsic :: iso_c_binding
            end module
            ",
        );
        let expected = vec![FortitudeViolation::from_start_end_line_col(
            UseAll {}.message(),
            &source,
            3,
            4,
            3,
            35,
        )];
        let rule = UseAll::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
