use crate::parser::fortran_language;
use crate::rules::{Code, Violation};
/// Defines rules that raise errors if implicit typing is in use.
use tree_sitter::{Node, Query};

pub const USE_IMPLICIT_NONE_MODULES_AND_PROGRAMS: &str = "\
    'implicit none' should be used in all modules and programs, as implicit typing
    reduces the readability of code and increases the chances of typing errors.";

pub fn use_implicit_none_modules_and_programs(
    code: Code,
    root: &Node,
    src: &str,
) -> Vec<Violation> {
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
        let query = Query::new(fortran_language(), query_txt.as_str()).unwrap();
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
                    violations.push(Violation::from_node(
                        &capture.node,
                        code,
                        format!("{} missing 'implicit none'", query_type).as_str(),
                    ));
                }
            }
        }
    }
    violations
}

pub const USE_IMPLICIT_NONE_INTERFACES: &str = "\
    Interface functions and subroutines require 'implicit none', even if they are inside
    a module that uses 'implicit none'.";

pub fn use_implicit_none_interfaces(code: Code, root: &Node, src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for query_type in ["function", "subroutine"] {
        let query_txt = format!(
            "(interface ({} (implicit_statement (none))? @implicit-none) @func)",
            query_type,
        );
        let query = Query::new(fortran_language(), query_txt.as_str()).unwrap();
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
                    violations.push(Violation::from_node(
                        &capture.node,
                        code,
                        format!("interface {} missing 'implicit none'", query_type).as_str(),
                    ));
                }
            }
        }
    }
    violations
}

pub const AVOID_SUPERFLUOUS_IMPLICIT_NONE: &str = "If a module has 'implicit none' set,
    it is not necessary to set it in contained functions and subroutines (except when
    using interfaces).";

pub fn avoid_superfluous_implicit_none(code: Code, root: &Node, src: &str) -> Vec<Violation> {
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
        let query = Query::new(fortran_language(), query_txt.as_str()).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();
        for match_ in cursor.matches(&query, *root, src.as_bytes()) {
            for capture in match_.captures {
                violations.push(Violation::from_node(
                    &capture.node,
                    code,
                    format!(
                        "'implicit none' is set on the enclosing {}, and isn't needed here",
                        query_type
                    )
                    .as_str(),
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
    fn test_module_and_program_missing_implicit_none() {
        let source = "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ";
        let expected_violations = [2, 6]
            .iter()
            .zip(["module", "program"])
            .map(|(line, kind)| {
                Violation::new(
                    *line,
                    TEST_CODE,
                    format!("{} missing 'implicit none'", kind).as_str(),
                )
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
        let source = "
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
            ";
        let expected_violations = [5, 14]
            .iter()
            .zip(["function", "subroutine"])
            .map(|(line, kind)| {
                Violation::new(
                    *line,
                    TEST_CODE,
                    format!("interface {} missing 'implicit none'", kind).as_str(),
                )
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
        let source = "
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
            ";
        let expected_violations = [6, 11, 24, 29]
            .iter()
            .zip(["module", "module", "program", "program"])
            .map(|(line, kind)| {
                Violation::new(
                    *line,
                    TEST_CODE,
                    format!(
                        "'implicit none' is set on the enclosing {}, and isn't needed here",
                        kind
                    )
                    .as_str(),
                )
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
