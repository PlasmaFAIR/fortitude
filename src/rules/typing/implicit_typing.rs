use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use tree_sitter::Node;
/// Defines rules that raise errors if implicit typing is in use.

fn implicit_statement_is_none(node: &Node) -> bool {
    if let Some(child) = node.child(1) {
        return child.kind() == "none";
    }
    false
}

fn child_is_implicit_none(node: &Node) -> bool {
    if let Some(child) = node.child_with_name("implicit_statement") {
        return implicit_statement_is_none(&child);
    }
    false
}

pub struct ImplicitTyping {}

impl Rule for ImplicitTyping {
    fn new(_settings: &Settings) -> Self {
        ImplicitTyping {}
    }

    fn explain(&self) -> &'static str {
        "
        'implicit none' should be used in all modules and programs, as implicit typing
        reduces the readability of code and increases the chances of typing errors.
        "
    }
}

impl ASTRule for ImplicitTyping {
    fn check(&self, node: &Node, _src: &str) -> Option<Vec<Violation>> {
        if !child_is_implicit_none(node) {
            let msg = format!("{} missing 'implicit none'", node.kind());
            return some_vec![Violation::from_node(msg, node)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["module", "submodule", "program"]
    }
}

pub struct InterfaceImplicitTyping {}

impl Rule for InterfaceImplicitTyping {
    fn new(_settings: &Settings) -> Self {
        InterfaceImplicitTyping {}
    }

    fn explain(&self) -> &'static str {
        "
        Interface functions and subroutines require 'implicit none', even if they are
        inside a module that uses 'implicit none'.
        "
    }
}

impl ASTRule for InterfaceImplicitTyping {
    fn check(&self, node: &Node, _src: &str) -> Option<Vec<Violation>> {
        let parent = node.parent()?;
        if parent.kind() == "interface" && !child_is_implicit_none(node) {
            let msg = format!("interface {} missing 'implicit none'", node.kind());
            return some_vec![Violation::from_node(msg, node)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}

pub struct SuperfluousImplicitNone {}

impl Rule for SuperfluousImplicitNone {
    fn new(_settings: &Settings) -> Self {
        SuperfluousImplicitNone {}
    }

    fn explain(&self) -> &'static str {
        "
        If a module has 'implicit none' set, it is not necessary to set it in contained
        functions and subroutines (except when using interfaces).
        "
    }
}

impl ASTRule for SuperfluousImplicitNone {
    fn check(&self, node: &Node, _src: &str) -> Option<Vec<Violation>> {
        if !implicit_statement_is_none(node) {
            return None;
        }
        let parent = node.parent()?;
        if matches!(parent.kind(), "function" | "subroutine") {
            for ancestor in parent.ancestors() {
                let kind = ancestor.kind();
                match kind {
                    "module" | "submodule" | "program" | "function" | "subroutine" => {
                        if child_is_implicit_none(&ancestor) {
                            let msg = format!("'implicit none' set on the enclosing {}", kind,);
                            return some_vec![Violation::from_node(msg, node)];
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

    fn entrypoints(&self) -> Vec<&'static str> {
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
        let rule = ImplicitTyping::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
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
        let rule = ImplicitTyping::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
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
        let rule = InterfaceImplicitTyping::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
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
        let rule = InterfaceImplicitTyping::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
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
            let msg = format!("'implicit none' set on the enclosing {}", kind);
            violation!(&msg, *line, *col)
        })
        .collect();
        let rule = SuperfluousImplicitNone::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
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
        let rule = SuperfluousImplicitNone::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
