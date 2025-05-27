use std::iter::once;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use anyhow::{Context, Result};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
enum StatType {
    Stat,
    IoStat,
    CmdStat,
}

enum CheckStatus {
    Checked,
    Unchecked,
    Overwritten,
}

// TODO Generalise to catch iostat and other error handling variables.
/// ## What does it do?
/// If the status of an `allocate` statement is checked by passing a variable to
/// the `stat` argument, that variable must be checked. To avoid confusing and
/// bug-prone control flow, the `stat` variable should be checked within the
/// same scope as the `allocate` statement.
///
/// ## Why is this bad?
/// By default, `allocate` statements will crash the program if the allocation
/// fails. This is often the desired behaviour, but to provide for cases in
/// which the user wants to handle allocation errors gracefully, they may
/// optionally check the status of an `allocate` statement by passing a variable
/// to the `stat` argument:
///
/// ```f90
/// allocate (x(BIG_INT), stat=status)
/// if (status /= 0) then
///   call handle_error(status)
/// end if
/// ```
///
/// However, if the `stat` variable is not checked, the program will continue to
/// run despite the allocation failure, which can lead to undefined behaviour.
#[derive(ViolationMetadata)]
pub(crate) struct UncheckedAllocateStat {
    name: String,
    stat: StatType,
    result: CheckStatus,
}

impl Violation for UncheckedAllocateStat {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, stat, result } = self;
        let check_status = match result {
            CheckStatus::Checked => "checked",
            CheckStatus::Unchecked => "not checked",
            CheckStatus::Overwritten => "overwritten before being checked",
        };
        format!("{stat} argument '{name}' is {check_status} in this scope.")
    }
}

impl AstRule for UncheckedAllocateStat {
    fn check(_settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = source.source_text();

        // Find a 'stat' argument in the allocate statement
        let stat_node = node.named_children(&mut node.walk()).find(|child| {
            child.kind() == "keyword_argument"
                && child
                    .child_by_field_name("name")
                    .is_some_and(|n| n.to_text(src) == Some("stat"))
        })?;

        let name = stat_node
            .child_by_field_name("value")?
            .to_text(src)?
            .to_string();

        // Check if the 'stat' variable is checked.
        //
        // - Scan all siblings of the allocate statement and their descendants
        //
        // - Find an instance of the variable being used somewhere.
        //
        // - If we reach the end of the siblings without finding one, try again
        //   from the sibling's ancestors. This is to cover cases where the allocate
        //   statement is nested something like an if statement, but the variable is
        //   checkout in the parent scope, e.g.:
        //
        // ```f90
        // if (twice_as_big) then
        //   allocate (x(2*BIG_INT), stat=status)
        // else
        //   allocate (x(BIG_INT), stat=status)
        // end if
        // if (status /= 0) then
        //   call handle_error(status)
        // end if
        // ```
        // - If we reach the end of the current function, subroutine, program, block,
        //   or module procedure without finding the variable, then we consider it a
        //   violation.
        //
        // - If the first use is in another allocate statement, or the left hand side
        //   of an assignment statement, consider that a violation regardless of other
        //   factors.

        for ancestor in once(*node)
            .chain(node.ancestors())
            .take_while(not_scope_boundary)
        {
            match find_stat_in_siblings(&ancestor, &name, src) {
                Ok(CheckStatus::Checked) => {
                    // Found the variable, so stop checking.
                    return None;
                }
                Ok(CheckStatus::Overwritten) => {
                    // Found the variable, but it has been overwritten.
                    return some_vec!(Diagnostic::from_node(
                        Self {
                            name,
                            stat: StatType::Stat,
                            result: CheckStatus::Overwritten
                        },
                        &stat_node
                    ));
                }
                Ok(CheckStatus::Unchecked) => {
                    // Didn't find it here. Continue searching.
                    continue;
                }
                _ => {
                    // Something went wrong, bail out.
                    return None;
                }
            }
        }
        some_vec!(Diagnostic::from_node(
            Self {
                name,
                stat: StatType::Stat,
                result: CheckStatus::Unchecked
            },
            &stat_node
        ))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["allocate_statement"]
    }
}

fn find_stat_in_siblings(node: &Node, stat_name: &str, src: &str) -> Result<CheckStatus> {
    let mut sibling = *node;

    while let Some(node) = sibling.next_sibling() {
        sibling = node;

        if new_branch(&sibling) {
            // Ignore occurences in else, elseif, and case statements,
            // as these are out of reach from the current assignment.
            continue;
        }

        if let Some(stat_node) = once(sibling)
            .chain(sibling.descendants())
            .find(|d| d.kind() == "identifier" && d.to_text(src) == Some(stat_name))
        {
            return stat_check_status(&stat_node, stat_name, src);
        }
    }
    Ok(CheckStatus::Unchecked)
}

fn stat_check_status(node: &Node, stat_name: &str, src: &str) -> Result<CheckStatus> {
    let ancestor = node.parent().context("Node should have a parent")?;

    // Two cases to consider:
    //
    // - The stat variable is on the left hand side of an assignment statement.
    // - The stat variable is passed to another error handling parameter.
    //
    // We expect false negatives if stat is passed to a user function or
    // subroutine and overwritten/ignored there.

    if ancestor.kind() == "assignment_statement" {
        if let Some(lhs) = ancestor.child_by_field_name("left") {
            if lhs.to_text(src).context("to_text error")? == stat_name {
                return Ok(CheckStatus::Overwritten);
            }
        }
    }
    if ancestor.kind() == "keyword_argument" {
        // TODO check that the next-highest ancestor is allocate, execute_command_line,
        // or an IO statement.
        if let Some(name_node) = ancestor.child_by_field_name("name") {
            if StatType::try_from(name_node.to_text(src).context("to_text error")?).is_ok() {
                return Ok(CheckStatus::Overwritten);
            }
        }
    }
    Ok(CheckStatus::Checked)
}

fn not_scope_boundary(node: &Node) -> bool {
    !matches!(
        node.kind(),
        "function_statement"
            | "subroutine_statement"
            | "program_statement"
            | "block_construct"
            | "module_procedure_statement"
    )
}

fn new_branch(node: &Node) -> bool {
    matches!(
        node.kind(),
        "elseif_clause" | "else_clause" | "case_statement"
    )
}
