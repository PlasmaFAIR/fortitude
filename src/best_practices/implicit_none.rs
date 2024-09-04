use crate::core::{Method, Rule, Violation};
use tree_sitter::{Node, Query};

/// Defines rules that raise errors if implicit typing is in use.

// Use implicit none in modules and programs
// -----------------------------------------

fn use_implicit_none_modules_and_programs(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for query_type in ["module", "submodule", "program"] {
        // Search for a module, submodule or program, and optionally an 'implicit none'.
        // Capture module, submodule or program in @mod, and 'implicit none' in @implicit-none.
        // We don't need @mod explicitly, but if @implicit-none isn't found, we'll need it
        // for error reporting.
        let query_txt = format!(
            "({} (implicit_statement (none))? @implicit-none) @mod",
            query_type
        );
        let query = Query::new(&tree_sitter_fortran::language(), query_txt.as_str()).unwrap();
        let implicit_none_index = query.capture_index_for_name("implicit-none").unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for captures in cursor
            .matches(&query, *root, src.as_bytes())
            .map(|x| x.captures)
        {
            // If the @implicit-none capture isn't found, record a violation.
            if !captures.iter().any(|&x| x.index == implicit_none_index) {
                // The only other captures must be the module, submodule, or program.
                for capture in captures {
                    let msg = format!("{} missing 'implicit none'", query_type);
                    violations.push(Violation::from_node(&msg, &capture.node));
                }
            }
        }
    }
    violations
}

pub struct UseImplicitNoneModulesAndPrograms {}

impl Rule for UseImplicitNoneModulesAndPrograms {
    fn method(&self) -> Method {
        Method::Tree(Box::new(use_implicit_none_modules_and_programs))
    }

    fn explain(&self) -> &str {
        "
        'implicit none' should be used in all modules and programs, as implicit typing
        reduces the readability of code and increases the chances of typing errors.
        "
    }
}

// Use implicit none in interfaces
// -------------------------------

fn use_implicit_none_interfaces(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for query_type in ["function", "subroutine"] {
        let query_txt = format!(
            "(interface ({} (implicit_statement (none))? @implicit-none) @func)",
            query_type,
        );
        let query = Query::new(&tree_sitter_fortran::language(), query_txt.as_str()).unwrap();
        let implicit_none_index = query.capture_index_for_name("implicit-none").unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for captures in cursor
            .matches(&query, *root, src.as_bytes())
            .map(|x| x.captures)
        {
            // If the @implicit-none capture isn't found, record a violation.
            if !captures.iter().any(|&x| x.index == implicit_none_index) {
                // The only other captures must be the module, submodule, or program.
                for capture in captures {
                    let msg = format!("interface {} missing 'implicit none'", query_type);
                    violations.push(Violation::from_node(&msg, &capture.node));
                }
            }
        }
    }
    violations
}

pub struct UseImplicitNoneInterfaces {}

impl Rule for UseImplicitNoneInterfaces {
    fn method(&self) -> Method {
        Method::Tree(Box::new(use_implicit_none_interfaces))
    }

    fn explain(&self) -> &str {
        "
        Interface functions and subroutines require 'implicit none', even if they are
        inside a module that uses 'implicit none'.
        "
    }
}

// Avoid implicit none where it isn't needed
// -----------------------------------------

fn avoid_superfluous_implicit_none(root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for query_type in ["module", "submodule", "program"] {
        let query_txt = format!(
            "({}
                (implicit_statement (none))
                (internal_procedures
                    (function (implicit_statement (none)) @x)*
                    (subroutine (implicit_statement (none)) @y)*
                )
            )",
            query_type
        );
        let query = Query::new(&tree_sitter_fortran::language(), query_txt.as_str()).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                let msg = format!(
                    "'implicit none' is set on the enclosing {}, and isn't needed here",
                    query_type
                );
                violations.push(Violation::from_node(&msg, &capture.node));
            }
        }
    }
    violations
}

pub struct AvoidSuperfluousImplicitNone {}

impl Rule for AvoidSuperfluousImplicitNone {
    fn method(&self) -> Method {
        Method::Tree(Box::new(avoid_superfluous_implicit_none))
    }

    fn explain(&self) -> &str {
        "
        If a module has 'implicit none' set, it is not necessary to set it in contained
        functions and subroutines (except when using interfaces).
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
    fn test_module_and_program_missing_implicit_none() {
        let source = dedent(
            "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ",
        );
        let expected_violations = [(2, 1, "module"), (6, 1, "program")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("{} missing 'implicit none'", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(
            use_implicit_none_modules_and_programs,
            source,
            Some(expected_violations),
        );
    }

    #[test]
    fn test_module_and_program_uses_implicit_none() {
        let source = "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function
            end module

            program my_program
                implicit none
                integer, paramter :: x = 2
                write(*,*) x
            end program
            ";
        test_tree_method(use_implicit_none_modules_and_programs, source, None);
    }

    #[test]
    fn test_interface_missing_implicit_none() {
        let source = dedent(
            "
            module my_module
                implicit none
                interface
                    integer function myfunc(x)
                        integer, intent(in) :: x
                    end function
                end interface
            end module

            program my_program
                implicit none
                interface
                    subroutine myfunc2(x)
                        integer, intent(inout) :: x
                    end subroutine
                end interface
                write(*,*) 42
            end program
            ",
        );
        let expected_violations = [(5, 9, "function"), (14, 9, "subroutine")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("interface {} missing 'implicit none'", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        test_tree_method(
            use_implicit_none_interfaces,
            source,
            Some(expected_violations),
        );
    }

    #[test]
    fn test_interface_uses_implicit_none() {
        let source = "
            module my_module
                implicit none
                interface
                    integer function myfunc(x)
                        implicit none
                        integer, intent(in) :: x
                    end function
                end interface
            end module

            program my_program
                implicit none
                interface
                    subroutine mysub(x)
                        implicit none
                        integer, intent(inout) :: x
                    end subroutine
                end interface
                write(*,*) 42
            end program
            ";
        test_tree_method(use_implicit_none_interfaces, source, None);
    }

    #[test]
    fn test_superflous_implicit_none() {
        let source = dedent(
            "
            module my_module
                implicit none
            contains
                integer function myfunc(x)
                    implicit none
                    integer, intent(in) :: x
                    myfunc = x * 2
                end function
                subroutine mysub(x)
                    implicit none
                    integer, intent(inout) :: x
                    x = x * 2
                end subroutine
            end module

            program my_program
                implicit none

                write(*,*) 42

            contains
                integer function myfunc2(x)
                    implicit none
                    integer, intent(in) :: x
                    myfunc2 = x * 2
                end function
                subroutine mysub2(x)
                    implicit none
                    integer, intent(inout) :: x
                    x = x * 2
                end subroutine
            end program
            ",
        );
        let expected_violations = [
            (6, 9, "module"),
            (11, 9, "module"),
            (24, 9, "program"),
            (29, 9, "program"),
        ]
        .iter()
        .map(|(line, col, kind)| {
            let msg = format!(
                "'implicit none' is set on the enclosing {}, and isn't needed here",
                kind
            );
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(
            avoid_superfluous_implicit_none,
            source,
            Some(expected_violations),
        );
    }

    #[test]
    fn test_non_superflous_implicit_none() {
        let source = "
            module my_module
                implicit none

                interface
                    integer function interfunc(x)
                        implicit none
                        integer, intent(in) :: x
                    end function
                end interface

            contains
                integer function myfunc(x)
                    integer, intent(in) :: x
                    myfunc = x * 2
                end function
                subroutine mysub(x)
                    integer, intent(inout) :: x
                    x = x * 2
                end subroutine
            end module

            program my_program
                implicit none

                write(*,*) 42

            contains
                integer function myfunc2(x)
                    integer, intent(in) :: x
                    myfunc2 = x * 2
                end function
                subroutine mysub2(x)
                    integer, intent(inout) :: x
                    x = x * 2
                end subroutine
            end program
            ";
        test_tree_method(avoid_superfluous_implicit_none, source, None);
    }
}
