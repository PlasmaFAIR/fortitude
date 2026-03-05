use crate::ast::{ControlFlow, FortitudeNode};
use crate::settings::CheckSettings;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode, SymbolTables};
use log::debug;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::TextRange;
use tree_sitter::Node;

/// ## What it does
/// Checks for uses of the obsolescent labelled `do` statements.
///
/// ## Why is this bad?
/// These statements were made completely redundant with the introduction of
/// construct names. Construct names are clearer and easier to understand, while
/// not allowing arbitrary `goto` statements and other confusing
///
/// The Fortran 2018 standard made these statements obsolescent,
///
/// ## Example
/// ```f90
///     do 10 i = 1, 10
///       foo(i) = i
/// 10  continue
/// ```
///
/// Use instead:
/// ```f90
///    do i = 1, 10
///      foo(i) = i
///    end do
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct LabelledDoLoop;

impl Violation for LabelledDoLoop {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Obsolescent labelled `do` loop".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Remove `do` label".to_string())
    }
}

impl AstRule for LabelledDoLoop {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // `do_label` is the obsolete node that we're trying to catch
        let label = node.child_by_field_name("do_label")?;
        let do_loop = node.parent()?;

        let mut diagnostic = Diagnostic::from_node(LabelledDoLoop {}, &label);
        if let Some(fix) = fix_labelled_do(&do_loop, &label, src) {
            diagnostic.set_fix(fix);
        }
        some_vec![diagnostic]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["do_statement"]
    }
}

/// ## What it does
/// Checks for `do` loops that share termination labels.
///
/// ## Why is this bad?
/// Labelled `do` loops that share statements to mark their end are particularly
/// confusing and bugprone, and were deleted in the Fortran 2018 standard.
///
/// ## Example
/// ```f90
///     do 10 i = 1, 10
///       do 10 j = 1, 10
///         foo(j, i) = i * j
/// 10  continue
/// ```
///
/// Use instead:
/// ```f90
///    do i = 1, 10
///      do j = 1, 10
///        foo(j, i) = i * j
///      end do
///    end do
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SharedDoTermination;

impl Violation for SharedDoTermination {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Shared `do` loop termination".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add `end do` statement".to_string())
    }
}

impl AstRule for SharedDoTermination {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // We only want to give one warning, so pick the node that has the
        // statement label. For multiple shared terminations, only first gets
        // the label, and the others get the following whitespace (something
        // about non-overlapping nodes).
        if node.to_text(src.source_text())?.is_empty() {
            return None;
        }

        // Mark the label + following statement
        let start = node.start_textsize();
        let end = node.next_non_comment_statement()?.end_textsize();
        let range = TextRange::new(start, end);

        let mut diagnostic = Diagnostic::new(Self {}, range);
        if let Some(fix) = fix_shared_termination(node, src) {
            diagnostic.set_fix(fix);
        }
        some_vec![diagnostic]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["do_label_virtual"]
    }
}


// Include whitespace and optional trailing comma
fn edit_remove_label(label_node: &Node) -> Option<Edit> {
    let end_edit = label_node.next_named_sibling()?.start_textsize();
    Some(Edit::deletion(label_node.start_textsize(), end_edit))
}

// TODO: needs Stylist for indentation/captialisation
fn fix_labelled_do(do_loop: &Node, label_node: &Node, source: &SourceFile) -> Option<Fix> {
    let src = source.source_text();
    let label_ref = label_node.to_text(src)?;
    debug!("label_ref = {label_ref:?}");

    // 0. check if shared termination label -> bail
    if do_loop.named_descendants().any(|child| {
        child.kind() == "do_label_virtual" && child.to_text(src).unwrap_or_default() == label_ref
    }) {
        debug!("** Can't fix, has shared termination");
        return None;
    }

    // 1. remove label reference
    let first_edit = edit_remove_label(label_node)?;

    let mut edits = Vec::new();

    // 2. check for gotos to label
    //    - yes -> need to keep label
    //    - no  -> remove label
    let keep_label = loop_has_branch_to_end(do_loop, label_ref, source);

    // 3. check termination statement
    //    - `end do` -> done
    //    - `continue` -> _replace_ with `end do`
    //    - anything else -> _add_ `end do` on following line
    //      - if keeping label, needing to move it to `end do`
    let end_statement = do_loop.children(&mut label_node.walk()).last()?;
    let end_do_label = end_statement.child_by_field_name("do_label")?;
    if end_statement.kind() != "end_do_label_loop_statement" {
        debug!("** Can't fix, didn't find correct end of loop");
        debug!("end_statement = {end_statement:?}");
        return None;
    }

    let end_label = end_do_label.to_text(src)?;
    if end_label != label_ref {
        debug!("** Can't fix, didn't find correct end of loop");
        debug!("end_statement = {end_statement:?}");
        debug!("do_label = {end_do_label:?}");
        debug!("end label ({end_label}) != start label ({label_ref})");
        return None;
    }

    let end_action = end_do_label.next_sibling()?;
    // A little gross, but even if we're keeping the label, we might need to
    // move it to a new statement
    let move_label = match end_action.kind() {
        "end" | "enddo" => false,
        "keyword_statement" if ControlFlow::maybe_from(&end_action, src)?.is_continue() => {
            edits.push(end_action.edit_replacement(source, "end do".to_string()));
            false
        }
        _ => {
            edits.push(add_new_end_do(&end_action, keep_label, end_label, source));
            true
        }
    };

    // Remove the label from the original end statement
    if !keep_label || move_label {
        let width = end_label.len();
        edits.push(end_do_label.edit_replacement(source, format!("{:width$}", " ")));
    }

    Some(Fix::unsafe_edits(first_edit, edits))
}

fn fix_shared_termination(do_label_virtual: &Node, source: &SourceFile) -> Option<Fix> {
    let src = source.source_text();

    let do_loop = do_label_virtual.parent()?.parent()?;
    let label_node = do_loop.child(0)?.child_by_field_name("do_label")?;
    let label_ref = label_node.to_text(src)?;

    if loop_has_branch_to_end(&do_loop, label_ref, source) {
        debug!("** Can't fix, contains branch to termination");
        return None;
    }

    let indentation = do_loop.indentation(source);
    let first_edit = edit_remove_label(&label_node)?;

    let end_statement = do_loop.children(&mut label_node.walk()).last()?;
    let end_do_label = end_statement.child_by_field_name("do_label")?;
    if end_statement.kind() != "end_do_label_loop_statement" {
        debug!("** Can't fix, didn't find correct end of loop");
        debug!("end_statement = {end_statement:?}");
        return None;
    }

    let end_label = end_do_label.to_text(src)?;
    if end_label != label_ref {
        debug!("** Can't fix, didn't find correct end of loop");
        debug!("end_statement = {end_statement:?}");
        debug!("do_label = {end_do_label:?}");
        debug!("end label ({end_label}) != start label ({label_ref})");
        return None;
    }

    let mut rest = Vec::new();
    let new_end_do = Edit::insertion(
        format!("{indentation}end do\n"),
        do_label_virtual.start_textsize(),
    );

    // Need to find the real end statement
    let end_action = find_real_end_do_loop(&do_loop)?;
    match end_action.kind() {
        "end" | "enddo" => rest.push(new_end_do),
        "keyword_statement" if ControlFlow::maybe_from(&end_action, src)?.is_continue() => {
            rest.push(new_end_do)
        }
        _ => {
            rest.push(Edit::insertion(format!("\n{indentation}end do"), end_action.end_textsize()));
            rest.push(add_new_end_do(&end_action, true, end_label, source));

            let width = end_label.len();
            rest.push(end_do_label.edit_replacement(source, format!("{:width$}", " ")));
        }
    };

    Some(Fix::safe_edits(first_edit, rest))
}

/// Given a `do_loop` that has a virtual end statement, find the enclosing
/// `do_loop` that has the actual end statement
fn find_real_end_do_loop<'a>(do_loop: &'a Node) -> Option<Node<'a>> {
    let mut end_loop = *do_loop;
    while let Some(current_end_action) = end_loop.children(&mut end_loop.walk()).last()?.child(0) {
        if current_end_action.kind() == "statement_label" {
            return current_end_action.next_sibling();
        }
        end_loop = end_loop.parent()?;
        // This should never happen
        if end_loop.kind() != "do_loop" {
            debug!("** left loop");
            return None;
        }
    }
    None
}

fn fix_bad_do_termination(
    node: &Node,
    end_action: &Node,
    end_do_label: &Node,
    source: &SourceFile,
) -> Option<Fix> {
    let do_loop = node.parent()?;
    let src = source.source_text();

    let end_label = end_do_label.to_text(source.source_text())?;

    // Empty label usually means this is the end of a shared termination chain.
    // But let's also double check
    if end_label.is_empty() || loop_has_shared_termination(&do_loop, end_label, src) {
        debug!("** Can't fix, has shared termination");
        return None;
    }

    let first_edit = add_new_end_do(end_action, true, end_label, source);

    let width = end_label.len();
    let edits = vec![end_do_label.edit_replacement(source, format!("{:width$}", " "))];

    Some(Fix::safe_edits(first_edit, edits))
}

fn loop_has_branch_to_end(do_loop: &Node, label_ref: &str, source: &SourceFile) -> bool {
    let src = source.source_text();
    do_loop.named_descendants().any(|child| {
        child
            .try_to_controlflow(source)
            .is_some_and(|control| control.goto_ref().is_some_and(|label| label == label_ref))
            || (child.kind() == "arithmetic_if_statement"
                && child.named_children(&mut child.walk()).any(|label| {
                    label.kind() == "statement_label_reference"
                        && label.to_text(src).is_some_and(|label| label == label_ref)
                }))
    })
}

fn loop_has_shared_termination(do_loop: &Node, label_ref: &str, src: &str) -> bool {
    do_loop.named_descendants().any(|child| {
        child.kind() == "do_label_virtual" && child.to_text(src).unwrap_or_default() == label_ref
    })
}

fn add_new_end_do(node: &Node, keep_label: bool, label: &str, source: &SourceFile) -> Edit {
    let moved_label = if keep_label {
        format!(" {label} ")
    } else {
        node.indentation_ignore_stmt_label(source)
    };

    Edit::insertion(format!("\n{moved_label}end do"), node.end_textsize())
}
