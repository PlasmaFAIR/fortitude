use std::iter::once;

use crate::ast::FortitudeNode;
use crate::diagnostics::{Annotation, Diagnostic, Span, Violation};
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};
use anyhow::{Context, Result, anyhow};
use fortitude_macros::{ViolationMetadata, field, kind};
use ruff_macros::derive_message_formats;
use ruff_text_size::TextRange;
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
        match node.kind_id() {
            kind!("allocate_statement") | kind!("deallocate_statement") => Ok(StatType::Stat),
            kind!("open_statement")
            | kind!("close_statement")
            | kind!("read_statement")
            | kind!("write_statement")
            | kind!("inquire_statement")
            | kind!("file_position_statement") => Ok(StatType::IoStat),
            kind!("subroutine_call") => {
                // Looking only for execute_command_line
                let subroutine_node = node
                    .child_by_field_id(field!("subroutine").into())
                    .context("Could not retrieve subroutine name")?;
                let subroutine_text = subroutine_node
                    .to_text(src)
                    .context("Failed to parse subroutine text")?;
                if "execute_command_line".eq_ignore_ascii_case(subroutine_text) {
                    Ok(StatType::CmdStat)
                } else {
                    Err(anyhow!("Unknown subroutine: {subroutine_text}"))
                }
            }
            _ => Err(anyhow!("Node does not have a stat type")),
        }
    }

    fn errmsg(self) -> &'static str {
        match self {
            StatType::Stat => "errmsg",
            StatType::IoStat => "iomsg",
            StatType::CmdStat => "cmdmsg",
        }
    }
}

enum CheckStatus {
    Checked,
    Unchecked,
    Overwritten(TextRange),
}

/// ## What does it do?
/// This rule detects whether a `stat`, `iostat`, and `cmdstat` argument is checked
/// within the same scope it is set.
///
/// ## Why is this bad?
/// By default, `allocate` statements will abort the program if the allocation
/// fails. This is often the desired behaviour, but to provide for cases in
/// which the developer wants to handle allocation errors gracefully, they may
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
            CheckStatus::Overwritten(_) => "overwritten before being checked",
        };
        format!("{stat} argument '{name}' is {check_status} in this scope.")
    }
}

impl AstRule for UncheckedStat {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let src = context.source_text();

        // Check this is an error checking statement, and get the stat type
        let stat_type = StatType::from_node(node, src).ok()?;
        let stat_name: &'static str = stat_type.into();

        // Find a 'stat' argument in the allocate statement
        let arg_list = if node.kind_id() == kind!("subroutine_call") {
            node.child_with_id(kind!("argument_list"))?
        } else {
            *node
        };
        let stat_node = arg_list.kwarg(stat_name, src)?;

        let name = stat_node
            .child_by_field_id(field!("value").into())?
            .to_text(src)?;

        // Check if the 'stat' variable is checked.
        //
        // - Scan all siblings of the allocate statement and their descendants
        //
        // - Find an instance of the variable being used somewhere.
        //
        // - If we reach the end of the siblings without finding one, try again
        //   from the sibling's ancestors. This is to cover cases where the allocate
        //   statement is nested in something like an if statement, but the variable
        //   is checked in the parent scope, e.g.:
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
            match find_stat_in_siblings(&ancestor, name, src) {
                Ok(CheckStatus::Checked) => {
                    // Found the variable, so stop checking.
                    return None;
                }
                Ok(CheckStatus::Overwritten(textrange)) => {
                    // Found the variable, but it has been overwritten.
                    let mut diagnostic = context.create_diagnostic(
                        Self {
                            name: name.to_owned(),
                            stat: stat_type,
                            result: CheckStatus::Overwritten(textrange),
                        },
                        stat_node,
                    );
                    diagnostic
                        .primary_annotation_mut()
                        .expect("Must have primary annotation")
                        .set_message("Set here...");
                    let annotation = Annotation::secondary(
                        Span::from(context.source_file().clone()).with_range(textrange),
                    )
                    .message("... and overwritten here");
                    diagnostic.annotate(annotation);
                    return some_vec!(diagnostic);
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
        some_vec!(context.create_diagnostic(
            Self {
                name: name.to_owned(),
                stat: stat_type,
                result: CheckStatus::Unchecked
            },
            stat_node
        ))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids![
            "allocate_statement",
            "deallocate_statement",
            "open_statement",
            "close_statement",
            "read_statement",
            "write_statement",
            "inquire_statement",
            "file_position_statement",
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

        if let Some(stat_node) = once(sibling).chain(sibling.descendants()).find(|d| {
            d.kind_id() == kind!("identifier")
                && d.to_text(src)
                    .is_some_and(|d| d.eq_ignore_ascii_case(stat_name))
        }) {
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

    if ancestor.kind_id() == kind!("assignment_statement")
        && let Some(lhs) = ancestor.child_by_field_id(field!("left").into())
    {
        let lhs_text = lhs.to_text(src).context("to_text error")?;
        if lhs_text.eq_ignore_ascii_case(stat_name) {
            return Ok(CheckStatus::Overwritten(lhs.textrange()));
        }
    }
    if ancestor.kind_id() == kind!("keyword_argument") {
        // See if the stat variable is passed to another error handling routine.
        let routine = ancestor
            .parent()
            .context("Keyword argument should have a parent")?;
        // If the parent is an argument list, then the routine is the grandparent.
        let routine = if routine.kind_id() == kind!("argument_list") {
            routine
                .parent()
                .context("Argument list should have a parent")?
        } else {
            routine
        };
        let is_in_error_checking_routine = StatType::from_node(&routine, src).is_ok();
        let kwarg_name_is_stat_type = StatType::try_from(
            ancestor
                .child_by_field_id(field!("name").into())
                .context("Keyword argument should have a name")?
                .to_text(src)
                .context("to_text error")?,
        )
        .is_ok();
        if is_in_error_checking_routine && kwarg_name_is_stat_type {
            return Ok(CheckStatus::Overwritten(ancestor.textrange()));
        }
    }
    Ok(CheckStatus::Checked)
}

fn not_scope_boundary(node: &Node) -> bool {
    !matches!(
        node.kind_id(),
        kind!("function")
            | kind!("subroutine")
            | kind!("program")
            | kind!("module_procedure")
            | kind!("block_construct")
    )
}

fn new_branch(node: &Node) -> bool {
    matches!(
        node.kind_id(),
        kind!("elseif_clause") | kind!("else_clause") | kind!("case_statement")
    )
}

enum AllocationType {
    Allocate,
    Deallocate,
}
impl AllocationType {
    fn from_node(node: &Node) -> Result<Self> {
        match node.kind_id() {
            kind!("allocate_statement") => Ok(AllocationType::Allocate),
            kind!("deallocate_statement") => Ok(AllocationType::Deallocate),
            _ => Err(anyhow!("Node is not an allocation type")),
        }
    }
}

/// ## What does it do?
/// This rule detects whether `stat` is used alongside multiple allocations or
/// deallocations.
///
/// ## Why is this bad?
/// When allocating or deallocating multiple variables at once, the use of a `stat`
/// parameter will permit the program to continue running even if one of the
/// allocations or deallocations fails. However, it may not be clear which
/// allocation or deallocation caused the error.
///
/// To avoid confusion, it is recommended to use separate allocate or deallocate
/// statements for each variable and check the `stat` parameters individually.
#[derive(ViolationMetadata)]
pub(crate) struct MultipleAllocationsWithStat {
    alloc_type: AllocationType,
}

impl Violation for MultipleAllocationsWithStat {
    #[derive_message_formats]
    fn message(&self) -> String {
        let allocations = match self.alloc_type {
            AllocationType::Allocate => "allocations",
            AllocationType::Deallocate => "deallocations",
        };
        format!("'stat' parameter used with multiple {allocations}.")
    }
}

impl AstRule for MultipleAllocationsWithStat {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let src = context.source_text();

        // Check this has a stat parameter
        let stat_node = node.kwarg("stat", src)?;

        // Count allocations
        let count = if node.kind_id() == kind!("allocate_statement") {
            count_allocations(node)
        } else {
            count_deallocations(node)
        };
        if count <= 1 {
            return None;
        }

        let alloc_type = AllocationType::from_node(node).ok()?;

        some_vec!(context.create_diagnostic(Self { alloc_type }, stat_node))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["allocate_statement", "deallocate_statement"]
    }
}

fn count_allocations(node: &Node) -> usize {
    node.children_by_field_id(field!("allocation"), &mut node.walk())
        .count()
}

fn count_deallocations(node: &Node) -> usize {
    node.named_children(&mut node.walk())
        .filter(|c| c.kind_id() == kind!("identifier"))
        .count()
}

/// ## What does it do?
/// This rule detects whether `stat` is used without also setting `errmsg` when
/// allocating or deallocating. Similarly checks for the use of `iostat` without
/// `iomsg` with IO routines, and `cmdstat` without `cmdmsg` when using
/// `execute_command_line`.
///
/// ## Why is this bad?
/// The error codes returned when using `stat`, `iostat`, or `cmdstat` are not
/// very informative on their own, and are not portable across compilers. It is
/// recommended to always capture the associated error message alongside the
/// error code:
///
/// ```f90
/// real, allocatable :: x(:)
/// integer :: status
/// character(256) :: message ! N.B. Can be allocatable in F2023+
///
/// allocate (x(100), stat=status, errmsg=message)
/// open (unit=10, file="data.txt", iostat=status, iomsg=message)
/// call execute_command_line("ls", cmdstat=status, cmdmsg=message)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct StatWithoutMessage {
    stat_type: StatType,
}

impl Violation for StatWithoutMessage {
    #[derive_message_formats]
    fn message(&self) -> String {
        let stat: &'static str = self.stat_type.into();
        let errmsg = self.stat_type.errmsg();
        format!("'{stat}' used without '{errmsg}'.")
    }
}

impl AstRule for StatWithoutMessage {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let src = context.source_text();

        let stat_type = StatType::from_node(node, src).ok()?;
        let stat_name: &'static str = stat_type.into();
        let arg_list = if node.kind_id() == kind!("subroutine_call") {
            node.child_with_id(kind!("argument_list"))?
        } else {
            *node
        };
        if let Some(kwarg) = arg_list.kwarg(stat_name, src)
            && !arg_list.kwarg_exists(stat_type.errmsg(), src)
        {
            return some_vec!(context.create_diagnostic(Self { stat_type }, kwarg));
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids![
            "allocate_statement",
            "deallocate_statement",
            "open_statement",
            "close_statement",
            "read_statement",
            "write_statement",
            "inquire_statement",
            "file_position_statement",
            "subroutine_call", // for execute_command_line
        ]
    }
}
