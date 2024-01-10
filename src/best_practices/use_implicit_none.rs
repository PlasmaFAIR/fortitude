use crate::parser::fortran_language;
use crate::rules;
/// Defines rules that raise errors if implicit typing is in use.
use tree_sitter::{Node, Query};

const CODE10: rules::Code = rules::Code::new(rules::Category::BestPractices, 10);
const MSG10: &str = "'implicit none' should be used in all modules and programs, as implicit \
    typing reduces the readability of code and increases the chances of typing errors.";

fn use_implicit_none_method(root: &Node, src: &str) -> Vec<rules::Violation> {
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
                    violations.push(rules::Violation::from_node(
                        &capture.node,
                        CODE10,
                        format!("{} missing 'implicit none'", query_type).as_str(),
                    ));
                }
            }
        }
    }
    violations
}

pub fn use_implicit_none() -> rules::Rule {
    rules::Rule::new(
        CODE10,
        rules::Method::Tree(use_implicit_none_method),
        MSG10,
        rules::Status::Standard,
    )
}

const CODE11: rules::Code = rules::Code::new(rules::Category::BestPractices, 11);
const MSG11: &str = "Interface functions and subroutines require 'implicit none', even if they \
    are inside a module that uses 'implicit none'.";

fn use_interface_implicit_none_method(root: &Node, src: &str) -> Vec<rules::Violation> {
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
                    violations.push(rules::Violation::from_node(
                        &capture.node,
                        CODE11,
                        format!("interface {} missing 'implicit none'", query_type).as_str(),
                    ));
                }
            }
        }
    }
    violations
}

pub fn use_interface_implicit_none() -> rules::Rule {
    rules::Rule::new(
        CODE11,
        rules::Method::Tree(use_interface_implicit_none_method),
        MSG11,
        rules::Status::Standard,
    )
}

const CODE12: rules::Code = rules::Code::new(rules::Category::BestPractices, 12);
const MSG12: &str = "If a module has 'implicit none' set, it is not necessary to set this in its \
    contained functions and subroutines (unless they're defined within interfaces).";

fn avoid_superfluous_implicit_none_method(root: &Node, src: &str) -> Vec<rules::Violation> {
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
        for captures in cursor
            .matches(&query, *root, src.as_bytes())
            .map(|x| x.captures)
        {
            for capture in captures {
                violations.push(rules::Violation::from_node(
                    &capture.node,
                    CODE12,
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

pub fn avoid_superfluous_implicit_none() -> rules::Rule {
    rules::Rule::new(
        CODE12,
        rules::Method::Tree(avoid_superfluous_implicit_none_method),
        MSG12,
        rules::Status::Standard,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::fortran_parser;

    fn test_helper(
        f: fn(&Node, &str) -> Vec<rules::Violation>,
        code: &str,
        err: Option<Vec<String>>,
    ) {
        let mut parser = fortran_parser();
        let tree = parser.parse(&code, None).unwrap();
        let root = tree.root_node();
        let rule = use_implicit_none();
        let mut violations = f(&root, code);
        violations.sort();
        match err {
            Some(x) => {
                assert_eq!(violations.len(), x.len());
                for (actual, expected) in violations.iter().zip(x) {
                    assert_eq!(actual.to_string(), expected);
                }
            }
            None => {
                // Do nothing!
            }
        }
    }

    #[test]
    fn test_missing_implicit_none() {
        let code = "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ";
        let errs = [2, 6]
            .iter()
            .zip(["module", "program"])
            .map(|(line, msg)| {
                format!("Line {}: B010 {} missing 'implicit none'", line, msg,).to_string()
            })
            .collect();
        test_helper(use_implicit_none_method, code, Some(errs));
    }

    #[test]
    fn test_uses_implicit_none() {
        let code = "
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
        test_helper(use_implicit_none_method, code, None);
    }

    #[test]
    fn test_interface_missing_implicit_none() {
        let code = "
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
        let errs = [5, 14]
            .iter()
            .zip(["function", "subroutine"])
            .map(|(line, msg)| {
                format!(
                    "Line {}: B011 interface {} missing 'implicit none'",
                    line, msg,
                )
                .to_string()
            })
            .collect();
        test_helper(use_interface_implicit_none_method, code, Some(errs));
    }

    #[test]
    fn test_interface_uses_implicit_none() {
        let code = "
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
        test_helper(use_interface_implicit_none_method, code, None);
    }

    #[test]
    fn test_superflous_implicit_none() {
        let code = "
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
        let errs = [6, 11, 24, 29]
            .iter()
            .zip(["module", "module", "program", "program"])
            .map(|(line, msg)| {
                format!(
                    "Line {}: B012 'implicit none' is set on the enclosing {}, and isn't needed here",
                    line, msg,
                )
                .to_string()
            })
            .collect();
        test_helper(avoid_superfluous_implicit_none_method, code, Some(errs));
    }

    #[test]
    fn test_non_superflous_implicit_none() {
        let code = "
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
        test_helper(avoid_superfluous_implicit_none_method, code, None);
    }
}
