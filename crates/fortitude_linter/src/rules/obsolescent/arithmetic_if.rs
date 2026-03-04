use crate::ast::{ControlFlow, ControlFlowNode, FortitudeNode};
use crate::settings::CheckSettings;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode, SymbolTables};
use itertools::Itertools;
use log::debug;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use strum_macros::{Display, EnumString};
use tree_sitter::Node;

/// ## What it does
/// Checks for arithmetic `if` statements.
///
/// ## Why is this bad?
/// The arithmetic `if` statement is used to jump between one of three statement
/// labels depending on whether the condition is below, above, or equal to
/// zero. However, this is incompatible with the IEEE 754 standard on floating
/// point numbers (due to the comparison between `real`s), and the use of
/// statment labels can hinder optimisation, as well as making the code harder
/// to read and maintain.
///
/// ## Example
/// ```f90
///     IF(x(1)) 10, 20, 30
/// 10  PRINT *, 'first case'
///     GOTO 40
/// 20  PRINT *, 'second case'
///     GOTO 40
/// 30  PRINT *, 'third case'
/// 40  CONTINUE
/// ```
///
/// Use instead:
/// ```f90
/// if (x(1) < 0) then
///   print*, "first case"
/// else if (x(1) > 0) then
///   print*, "third case"
/// else
///   print*, "second case"
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct ArithmeticIf;

impl Violation for ArithmeticIf {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Obsolete arithmetic `if`".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `if` statement or `if` construct".into())
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString)]
enum LabelRefCmp {
    #[strum(serialize = "<")]
    LessThan,
    #[strum(serialize = "<=")]
    LessThanEqual,
    #[strum(serialize = "==")]
    Equal,
    #[strum(serialize = "/=")]
    NotEqual,
    #[strum(serialize = ">=")]
    GreaterThanEqual,
    #[strum(serialize = ">")]
    GreaterThan,
}

/// There are either 2 or 3 distinct labels, and the transformed `if` can have
/// 1, 2, or 3 blocks
///
/// More precisely, we can have:
/// - 1 block: `if` -- 2 labels
/// - 2 blocks: `if/else` -- 2 labels
/// - 2 blocks: `if/else if` -- 3 labels
/// - 3 blocks: `if/else if/else` -- 3 labels
///
/// We get an `else` block if the (n-1)th block has a `goto` to a later sibling node.
/// In all other cases, we don't need an `else:
/// - fall-through
///    - have to be careful, we can't fix some of these cases
/// - ends in a `keyword_statement` (except above case)
///    - these are always controlflow redirection, e.g. `return` or other `goto`
fn fix_arithmetic_if(node: &Node, source: &SourceFile) -> Option<Fix> {
    let src = source.source_text();

    // First, we get the three label references
    let mut cursor = node.walk();
    let mut refs = node
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "statement_label_reference");

    let less_than = refs.next()?.to_text(src)?;
    let equal = refs.next()?.to_text(src)?;
    let greater_than = refs.next()?.to_text(src)?;

    // Check if the `equal` branch is the same as one of the others.
    // Probably don't need to worry about the other two being identical?
    let less_than_equal = less_than == equal;
    let greater_than_equal = greater_than == equal;
    let just_equal = less_than == greater_than;

    // Then we find the labels they reference...
    let mut refs = Vec::new();
    if less_than_equal {
        refs.push((less_than, LabelRefCmp::LessThanEqual));
    } else if !just_equal {
        refs.push((less_than, LabelRefCmp::LessThan));
    }

    if !less_than_equal && !greater_than_equal {
        refs.push((equal, LabelRefCmp::Equal));
    }

    if greater_than_equal {
        refs.push((greater_than, LabelRefCmp::GreaterThanEqual));
    } else if !just_equal {
        refs.push((greater_than, LabelRefCmp::GreaterThan));
    }

    if just_equal {
        // Whether we use `==` or `/=` is determined by which label appears
        // first, so put `/=` into the list and sort them
        refs.push((less_than, LabelRefCmp::NotEqual));
    }

    // ...and get them in the order that they appear
    let mut labels = refs
        .iter()
        .filter_map(|label| Some((node.next_statement_label(label.0, src)?, label.1)))
        .sorted_by_key(|label| label.0.start_byte());

    if labels.len() < 2 {
        // This shouldn't happen!
        debug!("** Can't fix because less than two sections");
        return None;
    }

    // We now definitely have at least two label targets, and possibly a third
    let first = labels.next()?;
    let second = labels.next()?;
    let third = labels.next();

    // First labelled section must be immediately following the `if` statement
    if first.0.prev_non_comment_sibling() != Some(*node) {
        debug!(
            "** Can't fix because first section is not immediately following `if`: {:?} != {node:?}\n",
            first.0.prev_non_comment_sibling(),
        );
        return None;
    }

    // TODO: this is too crude, need to just lop off brackets?
    let condition = node
        .child_with_name("parenthesized_expression")?
        .named_child(0)?
        .to_text(src)?;

    // TODO: indentation really needs Stylist
    // TODO: need to indent all lines in block?
    let base_indentation = node.indentation_ignore_stmt_label(source);

    // Replace the `if` statement itself
    let (end_size, whitespace) = end_of_replacement(&first.0, src, &base_indentation);
    let first_edit = Edit::replacement(
        format!("if ({condition} {} 0) then{whitespace}", first.1),
        node.start_textsize(),
        end_size,
    );

    // Rest of the edits
    let mut edits = Vec::new();

    // If we have only 2 distinct labels, then check if we need an else block,
    // and then we're done!
    if third.is_none() {
        // Check if we need an `else` block or not
        let end_node = if let Some(end_node) = get_end_of_else_block(&second.0, src) {
            // Replace second label target with `else`
            let (end_size, whitespace) = end_of_replacement(&second.0, src, &base_indentation);
            edits.push(Edit::replacement(
                format!("else{whitespace}"),
                second.0.prev_non_comment_sibling()?.start_textsize(),
                end_size,
            ));

            end_node
        } else {
            // No `else` block, replace second label target with `end if`
            second.0
        };

        edits.push(fix_end_if(&end_node, source, &base_indentation));
        return Some(Fix::unsafe_edits(first_edit, edits));
    }

    // Now definitely have a third label
    let third = third.unwrap();

    // Node that ends the first block
    let end_first = second
        .0
        .prev_non_comment_sibling()?
        .try_to_controlflow(source);
    // Node that ends the second block
    let end_second = third
        .0
        .prev_non_comment_sibling()?
        .try_to_controlflow(source);

    debug!("end_first = {end_first:?}");
    debug!("end_second = {end_second:?}");

    // If they're both gotos, then they better point to the same place
    if let (Some(end_first), Some(end_second)) = (&end_first, &end_second)
        && let Some(first_ref) = end_first.goto_ref()
        && let Some(second_ref) = end_second.goto_ref()
        && first_ref != second_ref
    {
        debug!("** Can't fix because gotos point to different places: {first_ref} != {second_ref}");
        return None;
    }

    // This is the node that should be replaced with `end if`. It's either the
    // target of the `goto` that ends the second block, or the third label
    // target -- or we can't fix this `if`
    let end_if_node = if let Some(final_block_last_node) = end_second {
        // Second block ends in a goto, so find its target
        match final_block_last_node.goto_ref() {
            Some(ref_) => second.0.next_statement_label_sibling(ref_, src),
            _ => Some(third.0),
        }
    } else if let Some(end_first) = end_first {
        match end_first.goto_ref() {
            Some(ref_) => {
                // First block ends in a goto that points to the third target
                if ref_ == third.0.to_text(src)? {
                    Some(third.0)
                } else {
                    None
                }
            }
            _ => Some(third.0),
        }
    } else {
        None
    };

    let end_if_node = if let Some(end_if_node) = end_if_node {
        end_if_node
    } else {
        debug!("** Can't fix, don't know end_if_node");
        return None;
    };

    // Replace the second label target
    let (start_size, else_) = start_of_replacement(&second.0, src, &base_indentation);
    let (end_size, whitespace) = end_of_replacement(&second.0, src, &base_indentation);
    edits.push(Edit::replacement(
        format!("{else_}if ({condition} {} 0) then{whitespace}", second.1),
        start_size,
        end_size,
    ));

    // Change the third label target to an `else` block if needed
    if end_if_node.start_textsize() > third.0.end_textsize() {
        let (start_size, _) = start_of_replacement(&third.0, src, &base_indentation);
        let (end_size, whitespace) = end_of_replacement(&third.0, src, &base_indentation);
        edits.push(Edit::replacement(
            format!("else{whitespace}"),
            start_size,
            end_size,
        ));
    }

    // Get the _next_ node after the close so we can eat any preceeding whitespace
    edits.push(fix_end_if(&end_if_node, source, &base_indentation));

    Some(Fix::unsafe_edits(first_edit, edits))
}

fn fix_end_if(node: &Node, source: &SourceFile, base_indentation: &str) -> Edit {
    let src = source.source_text();
    let (end_size, whitespace) = end_of_replacement(node, src, base_indentation);
    let start_byte = node.start_textsize();
    let start_index = source.to_source_code().line_index(start_byte);
    let start_line = source.to_source_code().line_start(start_index);
    debug!("start_line = {start_line:?}");
    Edit::replacement(
        format!("{base_indentation}end if{whitespace}"),
        start_line,
        end_size,
    )
}

/// Get the node that closes an `else` block.
///
/// If ``node`` is a ``goto`` and its target is a sibling, get the node of its
/// target.
fn get_end_of_else_block<'a>(node: &'a Node, src: &str) -> Option<Node<'a>> {
    let prev_node = node.prev_non_comment_sibling()?;
    let control = ControlFlow::maybe_from(&prev_node, src)?;
    match control {
        ControlFlow::GoTo(ref_) => node.next_statement_label_sibling(ref_, src),
        _ => None,
    }
}

/// Find start of replacement, and if we should close the block or use `else`
fn start_of_replacement(node: &Node, src: &str, base_indentation: &str) -> (TextSize, String) {
    // We should always have a previous node at this point!
    // TODO: This will consume any interleaving comments!
    let prev_node = node.prev_non_comment_sibling().unwrap();
    let control = ControlFlow::maybe_from(&prev_node, src);
    match control {
        Some(ControlFlow::GoTo(_)) | Some(ControlFlow::Continue) => {
            (prev_node.start_textsize(), "else ".to_string())
        }
        _ => (
            node.start_textsize(),
            format!("{base_indentation}end if\n{base_indentation}"),
        ),
    }
}

/// Find where we need to replace upto, and what whitespace is needed
fn end_of_replacement(node: &Node, src: &str, base_indentation: &str) -> (TextSize, String) {
    let next_node = node.next_sibling().unwrap();
    if let Some(control) = ControlFlowNode::maybe_from(next_node, src)
        && control.control_flow().is_continue()
    {
        // We've got a `continue` node that we can eat up to the end of
        (control.node().end_textsize(), "".to_string())
    } else {
        (next_node.start_textsize(), format!("\n{base_indentation}"))
    }
}

impl AstRule for ArithmeticIf {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let mut diagnostic = Diagnostic::from_node(ArithmeticIf {}, node);
        if let Some(fix) = fix_arithmetic_if(node, src) {
            diagnostic.set_fix(fix);
        }
        some_vec![diagnostic]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["arithmetic_if_statement"]
    }
}
