use crate::core::{Method, Rule, Violation};
use tree_sitter::{Node, Query};

/// Defines rules that check whether functions and subroutines are defined within modules,
/// submodules, or interfaces. Also ensures that `use` statements are used correctly.

// Define functions and subroutines in modules
// -------------------------------------------

fn use_modules_and_programs(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let query_txt = "(translation_unit [(function) @func (subroutine) @sub])";
    let query = Query::new(&tree_sitter_fortran::language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let node = capture.node;
            let msg = format!(
                "{} not contained within (sub)module or program",
                node.kind()
            );
            violations.push(Violation::from_node(&msg, &node));
        }
    }
    violations
}

pub struct UseModulesAndPrograms {}

impl Rule for UseModulesAndPrograms {
    fn method(&self) -> Method {
        Method::Tree(Box::new(use_modules_and_programs))
    }

    fn explain(&self) -> &str {
        "
        Functions and subroutines should be contained within (sub)modules or programs.
        Fortran compilers are unable to perform type checks and conversions on functions
        defined outside of these scopes, and this is a common source of bugs.
        "
    }
}

// Always follow 'use' with 'only'
// -------------------------------

fn use_only_clause(root: &Node, src: &str) -> Vec<Violation> {
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

pub struct UseOnlyClause {}

impl Rule for UseOnlyClause {
    fn method(&self) -> Method {
        Method::Tree(Box::new(use_only_clause))
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
    fn test_function_not_in_module() {
        let source = dedent(
            "
            integer function double(x)
              integer, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              integer, intent(inout) :: x
              x = 3 * x
            end subroutine
            ",
        );
        let expected_violations = [(2, 1, "function"), (7, 1, "subroutine")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("{} not contained within (sub)module or program", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(use_modules_and_programs, source, Some(expected_violations));
    }

    #[test]
    fn test_function_in_module() {
        let source = "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function

                subroutine triple(x)
                  integer, intent(inout) :: x
                  x = 3 * x
                end subroutine
            end module
            ";
        test_tree_method(use_modules_and_programs, source, None);
    }

    #[test]
    fn test_use_only_clause() {
        let source = dedent(
            "
            module my_module
                use, intrinsic :: iso_fortran_env, only: real32
                use, intrinsic :: iso_c_binding
            end module
            ",
        );
        let violation = violation!("'use' statement missing 'only' clause", 4, 5);
        test_tree_method(use_only_clause, source, Some(vec![violation]));
    }
}
