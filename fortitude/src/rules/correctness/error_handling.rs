use std::iter::once;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use anyhow::{anyhow, Context, Result};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
enum StatType {
    Stat,
    IoStat,
    CmdStat,
}

impl StatType {
    fn from_node(node: &Node, src: &str) -> Result<Self> {
        let node_kind = match node.kind() {
            "call_expression" => {
                // For call expressions, we need to check the name of the function.
                let identifier_node = node.child(0).context("Could not retrieve routine name")?;
                let identifier_text = identifier_node
                    .to_text(src)
                    .context("Failed to parse identifier text")?
                    .to_lowercase();
                match identifier_text.as_str() {
                    "deallocate" => "stat",
                    "wait" | "flush" => "iostat",
                    _ => return Err(anyhow!("Unknown routine: {identifier_text}")),
                }
            }
            "subroutine_call" => {
                // Looking only for execute_command_line
                let subroutine_node = node
                    .child_by_field_name("subroutine")
                    .context("Could not retrieve subroutine name")?;
                let subroutine_text = subroutine_node
                    .to_text(src)
                    .context("Failed to parse subroutine text")?
                    .to_lowercase();
                if subroutine_text == "execute_command_line" {
                    "cmdstat"
                } else {
                    return Err(anyhow!("Unknown subroutine: {subroutine_text}"));
                }
            }
            some_string => some_string,
        };
        match node_kind {
            "allocate_statement" | "stat" => Ok(StatType::Stat),
            "open_statement"
            | "close_statement"
            | "read_statement"
            | "write_statement"
            | "inquire_statement"
            | "file_position_statement"
            | "iostat" => Ok(StatType::IoStat),
            "cmdstat" => Ok(StatType::CmdStat),
            _ => Err(anyhow!("Node does not have a stat type")),
        }
    }
}

enum CheckStatus {
    Checked,
    Unchecked,
    Overwritten,
}

/// ## What does it do?
/// This rule detects whether a `stat`, `iostat`, and `cmdstat` variable is used
/// within the same scope it is set, and not overwritten or ignored.
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
/// Similar behaviour is exhibited by `deallocate` and IO statements such as
/// `open`, `read`, and `close`.
///
/// To avoid confusing and bug-prone control flow, the checks on status parameters
/// should occur within the same scope in which they are set.
#[derive(ViolationMetadata)]
pub(crate) struct UncheckedStat {
    name: String,
    stat: StatType,
    result: CheckStatus,
}

impl Violation for UncheckedStat {
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

impl AstRule for UncheckedStat {
    fn check(_settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = source.source_text();

        // Check this is an error checking statement, and get the stat type
        let stat_type = StatType::from_node(node, src).ok()?;
        let stat_name: &'static str = stat_type.into();

        // Find a 'stat' argument in the allocate statement
        let arg_list = if matches!(node.kind(), "subroutine_call" | "call_expression") {
            node.child_with_name("argument_list")?
        } else {
            *node
        };
        let stat_node = arg_list.named_children(&mut node.walk()).find(|child| {
            child.kind() == "keyword_argument"
                && child.child_by_field_name("name").is_some_and(|n| {
                    n.to_text(src).map(|s| s.to_lowercase()) == Some(stat_name.to_string())
                })
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
        vec![
            "allocate_statement",
            "open_statement",
            "close_statement",
            "read_statement",
            "write_statement",
            "inquire_statement",
            "file_position_statement",
            "call_expression", // various: deallocate, wait, flush
            "subroutine_call", // for execute_command_line
        ]
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
