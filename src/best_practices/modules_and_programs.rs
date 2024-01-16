use crate::parser::fortran_language;
use crate::rules::{Code, Violation};
use std::path::Path;
use tree_sitter::{Node, Query};
/// Defines rules that check whether functions and subroutines are defined within modules,
/// submodules, or interfaces. Also ensures that `use` statements are used correctly.

pub const USE_MODULES_AND_PROGRAMS: &str = "\
    Functions and subroutines should be contained within (sub)modules or programs.
    Fortran compilers are unable to perform type checks and conversions on functions
    defined outside of these scopes, and this is a common source of bugs.";

pub fn use_modules_and_programs(code: Code, path: &Path, root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let query_txt = "(translation_unit [(function) @func (subroutine) @sub])";
    let query = Query::new(fortran_language(), query_txt).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    for match_ in cursor.matches(&query, *root, src.as_bytes()) {
        for capture in match_.captures {
            let node = capture.node;
            violations.push(Violation::from_node(
                path,
                &node,
                code,
                format!(
                    "{} not contained within (sub)module or program",
                    node.kind(),
                )
                .as_str(),
            ));
        }
    }
    violations
}

pub const USE_ONLY_CLAUSE: &str = "\
    When using a module, it is recommended to add an 'only' clause to specify which
    components you intend to use:

    ```
    ! Not recommended
    use, intrinsic :: iso_fortran_env

    ! Better
    use, intrinsic :: iso_fortran_env, only: int32, real64
    ```

    This makes it easier for programmers to understand where the symbols in your code
    have come from, and avoids introducing many unneeded components to your local
    scope.";

const ONLY_CLAUSE_ERR: &str = "'use' statement missing 'only' clause";

pub fn use_only_clause(code: Code, path: &Path, root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    // Search for 'use' clause, and optionally an 'only' clause, capturing both.
    let query_txt = "(use_statement (included_items)? @only) @use";
    let query = Query::new(fortran_language(), query_txt).unwrap();
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
                violations.push(Violation::from_node(
                    path,
                    &capture.node,
                    code,
                    ONLY_CLAUSE_ERR,
                ));
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::{test_tree_method, TEST_CODE};

    #[test]
    fn test_function_not_in_module() {
        let source = "
            integer function double(x)
              integer, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              integer, intent(inout) :: x
              x = 3 * x
            end subroutine
            ";
        let path = Path::new("file.f90");
        let expected_violations = [2, 7]
            .iter()
            .zip(["function", "subroutine"])
            .map(|(line, kind)| {
                Violation::new(
                    &path,
                    *line,
                    TEST_CODE,
                    format!("{} not contained within (sub)module or program", kind).as_str(),
                )
            })
            .collect();
        test_tree_method(
            use_modules_and_programs,
            &path,
            source,
            Some(expected_violations),
        );
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
        let path = Path::new("file.f90");
        test_tree_method(use_modules_and_programs, &path, source, None);
    }

    #[test]
    fn test_use_only_clause() {
        let source = "
            module my_module
                use, intrinsic :: iso_fortran_env, only: real32
                use, intrinsic :: iso_c_binding
            end module
            ";
        let path = Path::new("file.f90");
        let violation = Violation::new(&path, 4, TEST_CODE, ONLY_CLAUSE_ERR);
        test_tree_method(use_only_clause, &path, source, Some(vec![violation]));
    }
}
