use crate::{Method, Rule, Violation};
use tree_sitter::{Node, Query};

/// Defines rules that check whether `use` statements are used correctly.

// TODO Check that 'used' entity is actually used somewhere

fn use_all(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    // Search for 'use' clause, and optionally an 'only' clause, capturing both.
    let query_txt = "(use_statement (included_items)? @only) @use";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let only_index = query.capture_index_for_name("only").unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for captures in cursor
        .matches(&query, *root, src.as_bytes())
        .map(|x| x.captures)
    {
        // If the @only capture isn't found, record a violation.
        if !captures.iter().any(|&x| x.index == only_index) {
            // The only remaining capture must be the use clause
            for capture in captures {
                let msg = "'use' statement missing 'only' clause";
                violations.push(Violation::from_node(msg, &capture.node));
            }
        }
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
