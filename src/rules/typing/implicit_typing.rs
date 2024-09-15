use crate::parsing::child_with_name;
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

fn implicit_none_not_found(node: &Node) -> Option<Violation> {
    if !child_is_implicit_none(node) {
        let msg = format!("{} missing 'implicit none'", node.kind());
        return Some(Violation::from_node(&msg, node));
    }
    None
}

fn implicit_typing(node: &Node, _src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "module" | "submodule" | "program" => {
                if let Some(x) = implicit_none_not_found(&child) {
                    violations.push(x)
                }
            }
            _ => (),
        }
        violations.extend(implicit_typing(&child, _src));
    }
    violations
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
}

fn interface_implicit_none_not_found(node: &Node) -> Option<Violation> {
    // check it is an interface function
    let parent = node.parent()?;
    if parent.kind() == "interface" && !child_is_implicit_none(node) {
        let msg = format!("interface {} missing 'implicit none'", node.kind());
        return Some(Violation::from_node(&msg, node));
    }
    None
}

fn interface_implicit_typing(node: &Node, _src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let kind = child.kind();
        if kind == "function" || kind == "subroutine" {
            if let Some(x) = interface_implicit_none_not_found(&child) {
                violations.push(x)
            }
        }
        violations.extend(interface_implicit_typing(&child, _src));
    }
    violations
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
}

fn top_level_scope(node: Node) -> Option<Node> {
    let parent = node.parent()?;
    match parent.kind() {
        "module" | "submodule" | "program" => Some(parent),
        _ => top_level_scope(parent),
    }
}

fn implicit_none_not_needed(node: &Node) -> Option<Violation> {
    if !implicit_statement_is_none(node) {
        return None;
    }
    let parent_kind = node.parent()?.kind();
    if parent_kind == "function" || parent_kind == "subroutine" {
        let enclosing = top_level_scope(*node)?;
        if child_is_implicit_none(&enclosing) {
            let msg = format!(
                "'implicit none' is set on the enclosing {}, and isn't needed here",
                enclosing.kind()
            );
            return Some(Violation::from_node(&msg, node));
        }
    }
    None
}

fn superfluous_implicit_none(node: &Node, _src: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let kind = child.kind();
        if kind == "implicit_statement" {
            if let Some(x) = implicit_none_not_needed(&child) {
                violations.push(x)
            }
        }
        violations.extend(superfluous_implicit_none(&child, _src));
    }
    violations
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::test_tree_method;
    use crate::violation;
    use textwrap::dedent;

    #[test]
    fn test_implicit_typing() {
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
        test_tree_method(implicit_typing, source, Some(expected_violations));
    }

    #[test]
    fn test_implicit_none() {
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
        test_tree_method(implicit_typing, source, None);
    }

    #[test]
    fn test_interface_implicit_typing() {
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
        test_tree_method(interface_implicit_typing, source, Some(expected_violations));
    }

    #[test]
    fn test_interface_implicit_none() {
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
        test_tree_method(interface_implicit_typing, source, None);
    }

    #[test]
    fn test_superfluous_implicit_none() {
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
                kind,
            );
            violation!(&msg, *line, *col)
        })
        .collect();
        test_tree_method(superfluous_implicit_none, source, Some(expected_violations));
    }

    #[test]
    fn test_no_superfluous_implicit_none() {
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
        test_tree_method(superfluous_implicit_none, source, None);
    }
}
