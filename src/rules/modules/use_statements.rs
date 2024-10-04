use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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
    fn check(&self, node: &Node, _src: &str) -> Option<Vec<Violation>> {
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
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_use_all() -> anyhow::Result<()> {
        let source = dedent(
            "
            module my_module
                use, intrinsic :: iso_fortran_env, only: real32
                use, intrinsic :: iso_c_binding
            end module
            ",
        );
        let expected = vec![violation!("'use' statement missing 'only' clause", 4, 5)];
        let rule = UseAll::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
