use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Defines rules that check whether `use` statements are used correctly.

// TODO Check that 'used' entity is actually used somewhere

pub struct UseAll {}

impl Rule for UseAll {
    fn new(_settings: &Settings) -> Self {
        UseAll {}
    }

    fn explain(&self) -> &'static str {
        "
        When using a module, it is recommended to add an 'only' clause to specify which
        components you intend to use:

        ```
        ! Not recommended
        use, intrinsic :: iso_fortran_env

        ! Better
        use, intrinsic :: iso_fortran_env, only: int32, real64
        ```

        This makes it easier for programmers to understand where the symbols in your
        code have come from, and avoids introducing many unneeded components to your
        local scope.
        "
    }
}

impl ASTRule for UseAll {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Violation>> {
        if node.child_with_name("included_items").is_none() {
            let msg = "'use' statement missing 'only' clause";
            return some_vec![Violation::from_node(msg, node)];
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
        let expected = vec![Violation::from_start_end_line_col(
            "'use' statement missing 'only' clause",
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
