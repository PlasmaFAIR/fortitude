use crate::ast::{ancestors, child_with_name};
use crate::{Method, Rule, Violation};
use tree_sitter::Node;
/// Defines rules that raise errors if implicit typing is in use.

fn implicit_statement_is_none(node: &Node) -> bool {
    if let Some(child) = node.child(1) {
        return child.kind() == "none";
    }
    false
}

fn child_is_implicit_none(node: &Node) -> bool {
    if let Some(child) = child_with_name(node, "implicit_statement") {
        return implicit_statement_is_none(&child);
    }
    false
}

fn implicit_typing(node: &Node, _src: &str) -> Option<Violation> {
    if !child_is_implicit_none(node) {
        let msg = format!("{} missing 'implicit none'", node.kind());
        return Some(Violation::from_node(&msg, node));
    }
    None
}

pub struct ImplicitTyping {}

impl Rule for ImplicitTyping {
    fn method(&self) -> Method {
        Method::Tree(implicit_typing)
    }

    fn explain(&self) -> &str {
        "
        'implicit none' should be used in all modules and programs, as implicit typing
        reduces the readability of code and increases the chances of typing errors.
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
        vec!["module", "submodule", "program"]
    }
}

fn interface_implicit_typing(node: &Node, _src: &str) -> Option<Violation> {
    let parent = node.parent()?;
    if parent.kind() == "interface" && !child_is_implicit_none(node) {
        let msg = format!("interface {} missing 'implicit none'", node.kind());
        return Some(Violation::from_node(&msg, node));
    }
    None
}

pub struct InterfaceImplicitTyping {}

impl Rule for InterfaceImplicitTyping {
    fn method(&self) -> Method {
        Method::Tree(interface_implicit_typing)
    }

    fn explain(&self) -> &str {
        "
        Interface functions and subroutines require 'implicit none', even if they are
        inside a module that uses 'implicit none'.
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
        vec!["function", "subroutine"]
    }
}

fn superfluous_implicit_none(node: &Node, _src: &str) -> Option<Violation> {
    if !implicit_statement_is_none(node) {
        return None;
    }
    let parent = node.parent()?;
    if matches!(parent.kind(), "function" | "subroutine") {
        for ancestor in ancestors(&parent) {
            let kind = ancestor.kind();
            match kind {
                "module" | "submodule" | "program" | "function" | "subroutine" => {
                    if child_is_implicit_none(&ancestor) {
                        let msg = format!(
                            "'implicit none' is set on the enclosing {}, can remove here",
                            kind,
                        );
                        return Some(Violation::from_node(&msg, node));
                    }
                }
                "interface" => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }
    None
}

pub struct SuperfluousImplicitNone {}

impl Rule for SuperfluousImplicitNone {
    fn method(&self) -> Method {
        Method::Tree(superfluous_implicit_none)
    }

    fn explain(&self) -> &str {
        "
        If a module has 'implicit none' set, it is not necessary to set it in contained
        functions and subroutines (except when using interfaces).
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
        vec!["implicit_statement"]
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
    fn test_implicit_typing() -> anyhow::Result<()> {
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
        let expected: Vec<Violation> = [(2, 1, "module"), (6, 1, "program")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("{} missing 'implicit none'", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        let actual = ImplicitTyping {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_implicit_none() -> anyhow::Result<()> {
        let source = dedent(
            "
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
            ",
        );
        let expected: Vec<Violation> = vec![];
        let actual = ImplicitTyping {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_interface_implicit_typing() -> anyhow::Result<()> {
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
        let expected: Vec<Violation> = [(5, 9, "function"), (14, 9, "subroutine")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("interface {} missing 'implicit none'", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        let actual = InterfaceImplicitTyping {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_interface_implicit_none() -> anyhow::Result<()> {
        let source = dedent(
            "
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
            ",
        );
        let expected: Vec<Violation> = vec![];
        let actual = InterfaceImplicitTyping {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_superfluous_implicit_none() -> anyhow::Result<()> {
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
        let expected: Vec<Violation> = [
            (6, 9, "module"),
            (11, 9, "module"),
            (24, 9, "program"),
            (29, 9, "program"),
        ]
        .iter()
        .map(|(line, col, kind)| {
            let msg = format!(
                "'implicit none' is set on the enclosing {}, can remove here",
                kind,
            );
            violation!(&msg, *line, *col)
        })
        .collect();
        let actual = SuperfluousImplicitNone {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_no_superfluous_implicit_none() -> anyhow::Result<()> {
        let source = dedent(
            "
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
            ",
        );
        let expected: Vec<Violation> = vec![];
        let actual = SuperfluousImplicitNone {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
