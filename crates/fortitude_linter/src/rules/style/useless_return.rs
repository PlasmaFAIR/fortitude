use crate::ast::{FortitudeNode, types::BlockExit};
use crate::fix::edits::redent;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode, Rule};
use log::debug;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::TextRange;
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
    const FIX_AVAILABILITY: ruff_diagnostics::FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `return` statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { branch } = self;
        Some(format!("Remove unnecessary '{branch}'"))
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
    const FIX_AVAILABILITY: ruff_diagnostics::FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `cycle` statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { branch } = self;
        Some(format!("Remove unnecessary '{branch}'"))
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
    const FIX_AVAILABILITY: ruff_diagnostics::FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch } = self;
        format!("Unecessary {branch} after `exit` statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { branch } = self;
        Some(format!("Remove unnecessary '{branch}'"))
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
    const FIX_AVAILABILITY: ruff_diagnostics::FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { branch, stop } = self;
        format!("Unecessary {branch} after `{stop}` statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { branch, .. } = self;
        Some(format!("Remove unnecessary '{branch}'"))
    }
}

pub(crate) fn check_superfluous_returns<'a>(
    settings: &CheckSettings,
    node: &'a Node,
    src: &'a SourceFile,
) -> Option<Diagnostic> {
    let text = node.child(0)?.to_text(src.source_text())?;
    let kind = BlockExit::try_from(text).ok()?;

    // Skip this node if it's inside an inline IF, because the rule does not apply
    if node.parent()?.inline_if_statement() {
        return None;
    }
    let sibling = node.next_non_comment_statement()?;
    let branch = match sibling.kind() {
        "else_clause" => "else",
        "elseif_clause" => "else-if",
        _ => return None,
    }
    .to_string();

    let mut diagnostic = match kind {
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
    };

    if let Some(ref mut diagnostic) = diagnostic
        && let Some(fix) = fix_superfluous_return(&sibling, src)
    {
        diagnostic.set_fix(fix);
    }

    diagnostic
}

fn fix_superfluous_return<'a>(branch: &'a Node, src: &'a SourceFile) -> Option<Fix> {
    // TODO(peter): use stylist for line endings and capitalisation

    let parent_if = branch.parent()?;
    if let Some(block_label) = parent_if.child_with_name("block_label_start_expression") {
        // TODO: This is overly cautious, we really only need to bail if this
        // branch contains a reference to the block label. If not, we could move
        // it from the original `end if` to the newly inserted one. We could
        // probably also handle it if the label _only_ appears in this branch
        let label_text = block_label
            .child(0)?
            .to_text(src.source_text())?
            .to_lowercase();
        debug!("Can't fix superfluous `else` due to block label '{label_text}'");
        return None;
    }

    let mut rest = Vec::new();

    let indentation = branch.indentation(src);

    let keyword = branch.child(0)?;
    let edit = if keyword.kind() == "elseif" {
        keyword.edit_replacement(src, format!("end if\n{indentation}if"))
    } else {
        // Indent the `if`
        if branch.kind() == "elseif_clause" {
            rest.push(Edit::replacement(
                indentation.clone(),
                keyword.end_textsize(),
                branch.child(1)?.start_textsize(),
            ));
        }

        // This covers both `else` and `else if`, replacing the `else`
        keyword.edit_replacement(src, "end if\n".to_string())
    };

    // for `else`, we need to also remove the existing `end if`, and then
    // unindent the block one level
    if branch.kind() == "else_clause" {
        let end_if = parent_if.named_children(&mut parent_if.walk()).last()?;
        rest.push(end_if.edit_delete(src));

        // Start reindenting from start of next line
        let lines = src.to_source_code();
        let else_line_num = lines.line_index(keyword.end_textsize());
        let next_line_num = else_line_num.saturating_add(1);

        let start = lines.line_start(next_line_num);
        let end = branch.children(&mut branch.walk()).last()?.end_textsize();
        let branch_range = TextRange::new(start, end);

        let branch_text = src.slice(branch_range);
        // Note that tabs, statement labels, and unindented comments will
        // probably mess things up in various ways!
        let replacement = redent(branch_text, &indentation);

        rest.push(Edit::range_replacement(replacement, branch_range));

        // There's a comment on the same line, fix up the whitespace
        if let Some(next_sib) = keyword.next_named_sibling()
            && next_sib.kind() == "comment"
            && else_line_num == lines.line_index(next_sib.start_textsize())
        {
            rest.push(Edit::replacement(
                indentation,
                keyword.end_textsize(),
                next_sib.start_textsize(),
            ));
        }
    };

    let fix = Fix::safe_edits(edit, rest);
    debug!("{fix:?}");

    Some(fix)
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
