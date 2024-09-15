use crate::parsing::child_with_name;
use crate::{Method, Rule, Violation};
use tree_sitter::Node;

/// Defines rules that check whether `use` statements are used correctly.

// TODO Check that 'used' entity is actually used somewhere

fn use_missing_only_clause(node: &Node) -> Option<Violation> {
    if child_with_name(node, "included_items").is_none() {
        let msg = "'use' statement missing 'only' clause";
        return Some(Violation::from_node(msg, node));
    }
    None
}

fn use_all(node: &Node, _src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "use_statement" {
            if let Some(x) = use_missing_only_clause(&child) {
                violations.push(x);
            }
        }
        violations.extend(use_all(&child, _src));
    }
    violations
}

pub struct UseAll {}

impl Rule for UseAll {
    fn method(&self) -> Method {
        Method::Tree(use_all)
    }

    fn explain(&self) -> &str {
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

    fn entrypoints(&self) -> Vec<&str> {
        vec!["use_statement"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::test_tree_method;
    use crate::violation;
    use textwrap::dedent;

    #[test]
    fn test_use_all() {
        let source = dedent(
            "
            module my_module
                use, intrinsic :: iso_fortran_env, only: real32
                use, intrinsic :: iso_c_binding
            end module
            ",
        );
        let violation = violation!("'use' statement missing 'only' clause", 4, 5);
        test_tree_method(use_all, source, Some(vec![violation]));
    }
}
