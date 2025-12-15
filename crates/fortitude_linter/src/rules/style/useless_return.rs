use crate::ast::{FortitudeNode, types::BlockExit};
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode, Rule};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for unnecessary `return` statements
///
/// ## Why is this bad?
/// Unlike many other languages, Fortran's `return` statement is only used to
/// return early from procedures, and not to return values. If a `return`
/// statement is the last executable statement in a procedure, it can safely be
/// removed.
///
/// ## Example
/// ```f90
/// integer function capped_add(a, b)
///   integer, intent(in) :: a, b
///   if ((a + b) > 10) then
///     capped_add = 10
///     return
///   end if
///   capped_add = a + b
///   return   ! This `return` statement does nothing
/// end function capped_add
/// ```
///
/// Use instead:
/// ```f90
/// integer function capped_add(a, b)
///   integer, intent(in) :: a, b
///   if ((a + b) > 10) then
///     capped_add = 10
///     return
///   end if
///   capped_add = a + b
/// end function capped_add
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UselessReturn;

impl AlwaysFixableViolation for UselessReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        "useless `return` statement`".to_string()
    }

    fn fix_title(&self) -> String {
        "remove `return` statement".to_string()
    }
}

impl AstRule for UselessReturn {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        if !node
            .to_text(src.source_text())?
            .eq_ignore_ascii_case("return")
        {
            return None;
        }
        let sibling = node.next_non_comment_sibling();
        if !matches!(
            sibling?.kind(),
            "end_function_statement" | "end_subroutine_statement" | "internal_procedures"
        ) {
            return None;
        }
        let edit = node.edit_delete(src);
        some_vec!(Diagnostic::from_node(Self, node).with_fix(Fix::safe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["keyword_statement"]
    }
}

/// ## What it does
/// Checks for `else` statements with a `return` statement in the preceeding
/// `if` block
///
/// ## Why is this bad?
/// The `else` statement is not needed as the `return` statement will always
/// exit the parent function. Removing the `else` will reduce nesting and make
/// the code more readable.
///
/// ## Example
/// ```f90
/// integer function max(a, b):
///   integer, intent(in) :: a, b
///   if (a > b) then
///     max = a
///     return
///   else
///     max = b
///   end if
/// end function max
/// ```
///
/// Use instead:
/// ```f90
/// integer function max(a, b):
///   integer, intent(in) :: a, b
///   if (a > b) then
///     max = a
///     return
///   end if
///   max = b
/// end function max
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousElseReturn {
    branch: String,
}

impl Violation for SuperfluousElseReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `return` statement")
    }
}

/// ## What it does
/// Checks for `else` statements with a `cycle` statement in the preceeding
/// `if` block
///
/// ## Why is this bad?
/// The `else` statement is not needed as the `cycle` statement will always
/// continue onto the next iteration of the loop. Removing the `else` will
/// reduce nesting and make the code more readable.
///
/// ## Example
/// ```f90
/// integer function foo(a, b):
///   integer, intent(in) :: a, b
///   integer :: i
///   do i = 1, a
///     if (b > i) then
///       cycle
///     else
///       foo = b
///     end if
///   end do
/// end function foo
/// ```
///
/// Use instead:
/// ```f90
/// integer function foo(a, b):
///   integer, intent(in) :: a, b
///   integer :: i
///   do i = 1, a
///     if (b > i) then
///       cycle
///     end if
///     foo = b
///   end do
/// end function foo
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousElseCycle {
    branch: String,
}

impl Violation for SuperfluousElseCycle {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `cycle` statement")
    }
}

/// ## What it does
/// Checks for `else` statements with a `exit` statement in the preceeding
/// `if` block
///
/// ## Why is this bad?
/// The `else` statement is not needed as the `exit` statement will always
/// exit the enclosing loop. Removing the `else` will reduce nesting and make
/// the code more readable.
///
/// ## Example
/// ```f90
/// integer function foo(a, b):
///   integer, intent(in) :: a, b
///   integer :: i
///   do i = 1, a
///     if (b > i) then
///       exit
///     else
///       foo = b
///     end if
///   end do
/// end function foo
/// ```
///
/// Use instead:
/// ```f90
/// integer function foo(a, b):
///   integer, intent(in) :: a, b
///   integer :: i
///   do i = 1, a
///     if (b > i) then
///       exit
///     end if
///     foo = b
///   end do
/// end function foo
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousElseExit {
    branch: String,
}

impl Violation for SuperfluousElseExit {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `exit` statement")
    }
}

/// ## What it does
/// Checks for `else` statements with a `stop` statement in the preceeding
/// `if` block
///
/// ## Why is this bad?
/// The `else` statement is not needed as the `stop` statement will always
/// exit the parent function. Removing the `else` will reduce nesting and make
/// the code more readable.
///
/// ## Example
/// ```f90
/// integer function max(a, b):
///   integer, intent(in) :: a, b
///   if (a > b) then
///     max = a
///     stop
///   else
///     max = b
///   end if
/// end function max
/// ```
///
/// Use instead:
/// ```f90
/// integer function max(a, b):
///   integer, intent(in) :: a, b
///   if (a > b) then
///     max = a
///     stop
///   end if
///   max = b
/// end function foo
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousElseStop {
    branch: String,
    stop: String,
}

impl Violation for SuperfluousElseStop {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch, stop } = self;
        format!("Unecessary {branch} after `{stop}` statement")
    }
}

pub(crate) fn check_superfluous_returns<'a>(
    settings: &CheckSettings,
    node: &'a Node,
    src: &'a SourceFile,
) -> Option<Diagnostic> {
    let text = node.child(0)?.to_text(src.source_text())?;
    let kind = BlockExit::try_from(text).ok()?;

    let sibling = node.next_non_comment_statement();
    let branch = match sibling?.kind() {
        "else_clause" => "else",
        "elseif_clause" => "else-if",
        _ => return None,
    }
    .to_string();

    match kind {
        BlockExit::Return => Diagnostic::from_node_if_rule_enabled(
            settings,
            Rule::SuperfluousElseReturn,
            SuperfluousElseReturn { branch },
            node,
        ),
        BlockExit::Cycle => Diagnostic::from_node_if_rule_enabled(
            settings,
            Rule::SuperfluousElseCycle,
            SuperfluousElseCycle { branch },
            node,
        ),
        BlockExit::Exit => Diagnostic::from_node_if_rule_enabled(
            settings,
            Rule::SuperfluousElseExit,
            SuperfluousElseExit { branch },
            node,
        ),
        BlockExit::Stop => Diagnostic::from_node_if_rule_enabled(
            settings,
            Rule::SuperfluousElseStop,
            SuperfluousElseStop {
                branch,
                stop: "stop".to_string(),
            },
            node,
        ),
        BlockExit::Error => Diagnostic::from_node_if_rule_enabled(
            settings,
            Rule::SuperfluousElseStop,
            SuperfluousElseStop {
                branch,
                stop: "error stop".to_string(),
            },
            node,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use tree_sitter::Parser;

    #[test]
    fn test_next_non_comment_sibling() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
! some comment
subroutine foo
end subroutine foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;
        let next = root
            .next_non_comment_sibling()
            .context("Failed to find next non-comment sibling")?;
        assert_eq!(next.kind(), "subroutine");

        Ok(())
    }

    #[test]
    fn test_next_non_comment_statement() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
subroutine foo
end subroutine foo
! comment
subroutine bar
end subroutine bar
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let first_end_sub = tree.root_node().child(0).unwrap().child(1).unwrap();

        assert_eq!(first_end_sub.kind(), "end_subroutine_statement");
        println!("{first_end_sub:?} {}", first_end_sub.to_text(code).unwrap());

        let next = first_end_sub
            .next_non_comment_statement()
            .context("Failed to find next non-comment statement")?;
        assert_eq!(next.kind(), "subroutine");
        let text = next
            .child(0)
            .unwrap()
            .child_by_field_name("name")
            .context("Missing name")?
            .to_text(code)
            .unwrap();
        assert_eq!(text, "bar");

        Ok(())
    }
}
