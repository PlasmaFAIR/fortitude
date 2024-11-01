use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, FromASTNode, Rule};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
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

/// ## What does it do?
/// Checks for missing `implicit none`
///
/// ## Why is this bad?
/// 'implicit none' should be used in all modules and programs, as implicit typing
/// reduces the readability of code and increases the chances of typing errors.
#[violation]
pub struct ImplicitTyping {
    entity: String,
}

impl Violation for ImplicitTyping {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity } = self;
        format!("{entity} missing 'implicit none'")
    }
}

impl Rule for ImplicitTyping {
    fn new(_settings: &Settings) -> Self {
        ImplicitTyping {
            entity: String::default(),
        }
    }
}
impl ASTRule for ImplicitTyping {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if !child_is_implicit_none(node) {
            let entity = node.kind().to_string();
            let block_stmt = node.child(0)?;
            return some_vec![Diagnostic::from_node(Self { entity }, &block_stmt)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["module", "submodule", "program"]
    }
}

/// ## What it does
/// Checks for missing `implicit none` in interfaces
///
/// ## Why is this bad?
/// Interface functions and subroutines require 'implicit none', even if they are
/// inside a module that uses 'implicit none'.
#[violation]
pub struct InterfaceImplicitTyping {
    name: String,
}

impl Violation for InterfaceImplicitTyping {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("interface '{name}' missing 'implicit none'")
    }
}

impl Rule for InterfaceImplicitTyping {
    fn new(_settings: &Settings) -> Self {
        InterfaceImplicitTyping {
            name: String::default(),
        }
    }
}

impl ASTRule for InterfaceImplicitTyping {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let parent = node.parent()?;
        if parent.kind() == "interface" && !child_is_implicit_none(node) {
            let name = node.kind().to_string();
            let interface_stmt = node.child(0)?;
            return some_vec![Diagnostic::from_node(Self { name }, &interface_stmt)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}

/// ## What it does
/// Checks for unnecessary `implicit none` in module procedures
///
/// ## Why is this bad?
/// If a module has 'implicit none' set, it is not necessary to set it in contained
/// functions and subroutines (except when using interfaces).
#[violation]
pub struct SuperfluousImplicitNone {
    entity: String,
}

impl Violation for SuperfluousImplicitNone {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity } = self;
        format!("'implicit none' set on the enclosing {entity}")
    }
}

impl Rule for SuperfluousImplicitNone {
    fn new(_settings: &Settings) -> Self {
        SuperfluousImplicitNone {
            entity: String::default(),
        }
    }
}

impl ASTRule for SuperfluousImplicitNone {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if !implicit_statement_is_none(node) {
            return None;
        }
        let parent = node.parent()?;
        if matches!(parent.kind(), "function" | "subroutine") {
            for ancestor in parent.ancestors() {
                let kind = ancestor.kind();
                match kind {
                    "module" | "submodule" | "program" | "function" | "subroutine" => {
                        if !child_is_implicit_none(&ancestor) {
                            continue;
                        }
                        let entity = kind.to_string();
                        return some_vec![Diagnostic::from_node(Self { entity }, node)];
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
    use crate::{settings::default_settings, test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_implicit_typing() -> anyhow::Result<()> {
        let source = test_file(
            "
            module my_module
                parameter(N = 1)
            end module

            program my_program
                write(*,*) 42
            end program
            ",
        );
        let expected: Vec<_> = [(0, 1, 0, 17, "module"), (5, 0, 5, 18, "program")]
            .iter()
            .map(|(start_line, start_col, end_line, end_col, kind)| {
                Diagnostic::from_start_end_line_col(
                    ImplicitTyping {
                        entity: kind.to_string(),
                    },
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            })
            .collect();
        let rule = ImplicitTyping::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_implicit_none() -> anyhow::Result<()> {
        let source = test_file(
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
        let expected: Vec<Diagnostic> = vec![];
        let rule = ImplicitTyping::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_interface_implicit_typing() -> anyhow::Result<()> {
        let source = test_file(
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
        let expected: Vec<_> = [(4, 8, 4, 34, "function"), (13, 8, 13, 29, "subroutine")]
            .iter()
            .map(|(start_line, start_col, end_line, end_col, kind)| {
                Diagnostic::from_start_end_line_col(
                    InterfaceImplicitTyping {
                        name: kind.to_string(),
                    },
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            })
            .collect();
        let rule = InterfaceImplicitTyping::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_interface_implicit_none() -> anyhow::Result<()> {
        let source = test_file(
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
        let expected: Vec<Diagnostic> = vec![];
        let rule = InterfaceImplicitTyping::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_superfluous_implicit_none() -> anyhow::Result<()> {
        let source = test_file(
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
        let expected: Vec<_> = [
            (5, 8, 5, 21, "module"),
            (10, 8, 10, 21, "module"),
            (23, 8, 23, 21, "program"),
            (28, 8, 28, 21, "program"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, kind)| {
            Diagnostic::from_start_end_line_col(
                SuperfluousImplicitNone {
                    entity: kind.to_string(),
                },
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = SuperfluousImplicitNone::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_no_superfluous_implicit_none() -> anyhow::Result<()> {
        let source = test_file(
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
        let expected: Vec<Diagnostic> = vec![];
        let rule = SuperfluousImplicitNone::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
