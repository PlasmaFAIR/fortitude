use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind, kw};
use fortitude_sitter::Node;
use fortitude_sitter::traits::TextRanged;
use ruff_macros::derive_message_formats;
use ruff_source_file::{LineEnding, find_newline};
use ruff_text_size::TextSize;

/// ## What it does
/// Checks for misleading semicolons in `if` statements.
///
/// ## Why is this bad?
/// The following code may appear to execute two statements only if the `if`
/// condition is true, but in actuality the second statement will always be
/// executed:
///
/// ```f90
/// if (condition) print *, "Hello"; print *, "World"
/// ```
///
/// It is equivalent to:
///
/// ```f90
/// if (condition) print *, "Hello"
/// print *, "World"
/// ```
///
/// Users should be cautious applying this fix. If the intent was to have
/// both statements execute only if the condition is true, then the user
/// should rewrite the code to use an `if` statement with a block:
///
/// ```f90
/// if (condition) then
///     print *, "Hello"
///     print *, "World"
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MisleadingInlineIfSemicolon {}

impl AlwaysFixableViolation for MisleadingInlineIfSemicolon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Semicolon following inline if-statement is misleading".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace with newline, or convert to `if(...) then` statement".to_string()
    }
}
impl AstRule for MisleadingInlineIfSemicolon {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // If this is an `if (...) then` construct, exit early
        if !node.inline_if_statement() {
            return None;
        }

        // Check that the if statement is followed directly by a semicolon
        if let Some(semicolon_node) = node.next_sibling().filter(|n| n.kind_id() == kw!(";")) {
            // Find the first following non-semicolon node, and check that it's
            // on the same line as the first semicolon node.
            // Comments on the same line are permitted.
            let mut current_node = semicolon_node;
            while let Some(next) = current_node.next_sibling() {
                if next.kind_id() == kw!(";") {
                    current_node = next;
                    continue;
                }
                if next.start_position().row != semicolon_node.start_position().row
                    || next.kind_id() == kind!("comment")
                {
                    return None;
                }
                break;
            }
            // Replace semicolon node with a newline to make it clear that the
            // second statement is not part of the if statement. We should also
            // take care to handle indentation and other whitespace.
            let start = semicolon_node.start_textsize();
            let mut end = semicolon_node.end_byte();
            let text = context.source_text().as_bytes();
            while text[end] == b' ' || text[end] == b'\t' {
                end += 1;
            }
            let end = TextSize::try_from(end).unwrap();
            let indentation = node.indentation();
            let edit = Edit::replacement(format!("\n{indentation}"), start, end);
            return some_vec!(
                context
                    .create_diagnostic(MisleadingInlineIfSemicolon {}, semicolon_node)
                    .with_fix(Fix::safe_edit(edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["if_statement"]
    }
}

/// ## What it does
/// Checks for misleading line continuations in inline `if` statements.
///
/// ## Why is this bad?
/// An inline `if` statement followed immediately by a line continuation
/// can be easily confused for a block `if` statement:
///
/// ```f90
/// if (condition) &
///     call a_very_long_subroutine_name(with, many, ..., arguments)
/// ```
///
/// If a developer wishes to add a second statement to the `if` 'block',
/// they may be tempted to write:
///
/// ```f90
/// if (condition) &
///     call a_very_long_subroutine_name(with, many, ..., arguments)
///     call another_subroutine(args, ...)  ! Always executes!
/// ```
///
/// To avoid this confusion, inline `if` statements that spill over multiple
/// lines should be written as an if-then-block:
///
/// ```f90
/// if (condition) then
///     call a_very_long_subroutine_name(with, many, ..., arguments)
///     call another_subroutine(args, ...)  ! Only executes if condition is true
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MisleadingInlineIfContinuation {}

impl AlwaysFixableViolation for MisleadingInlineIfContinuation {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Line continuation in inline if-statement is misleading".to_string()
    }

    fn fix_title(&self) -> String {
        "Convert to `if(...) then` block".to_string()
    }
}
impl AstRule for MisleadingInlineIfContinuation {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // If this is an `if (...) then` construct, exit early
        if !node.inline_if_statement() {
            return None;
        }

        // Check if the condition is immediately following by a continuation
        let body_start = node
            .child_with_id(kind!("parenthesized_expression"))?
            .next_sibling()?;
        if body_start.kind_id() == kw!("&") {
            let content = ifthenify(node)?;
            let start_byte = node.start_textsize();
            let end_byte = node.end_textsize();
            let edit = Edit::replacement(content, start_byte, end_byte);
            return some_vec!(
                context
                    .create_diagnostic(MisleadingInlineIfContinuation {}, node)
                    .with_fix(Fix::safe_edit(edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["if_statement"]
    }
}

/// Given an if statement node, convert it from a one-line if statement
/// to a block if statement.
/// Returns None if the node is already a block if statement.
pub fn ifthenify(node: &Node) -> Option<String> {
    // check that the node is an if statement without an end if statement
    if !node.inline_if_statement() {
        return None;
    }
    let text = node.text().trim();
    let bytes = text.as_bytes();
    // If CrLf line endings are used, be careful to keep them in the fix
    let nl = find_newline(text)
        .map(|(_, ending)| ending)
        .unwrap_or(LineEnding::Lf)
        .as_str();
    // Divide the if statement into everything up to the end of the condition
    // and the body.
    let condition_node = node.child_with_id(kind!("parenthesized_expression"))?;
    let condition_end_byte = condition_node.end_byte() - node.start_byte();
    let mut body_start_byte = condition_end_byte;
    // Skip over any whitespace or line continuations between the condition
    // and the body.
    while body_start_byte < node.end_byte()
        && (bytes[body_start_byte].is_ascii_whitespace() || bytes[body_start_byte] == b'&')
    {
        body_start_byte += 1;
    }
    // Build the new if statement.
    let prelude = &text[..condition_end_byte];
    let body = &text[body_start_byte..];
    let indentation = node.indentation();
    // Assume two-space indent for the if body
    Some(format!(
        "{prelude} then{nl}{indentation}  {body}{nl}{indentation}end if"
    ))
}

/// ## What it does
/// Checks for a constant `.true.` or `.false.` as an operand of `.and.` or `.or.`.
///
/// ## Why is this bad?
/// A constant operand either cannot change the result or fixes it, so the
/// expression does not test what it appears to test. Both shapes usually come
/// from an edit that left a placeholder behind or from a condition that was
/// rewritten by halves.
///
/// ## Example
/// ```f90
/// if (is_ready .or. .true.) then
///     call run()
/// end if
/// ```
/// Use instead:
/// ```f90
/// if (is_ready) then
///     call run()
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct ConstantLogicalOperand {
    operand: String,
    operator: String,
    literal: String,
}

impl Violation for ConstantLogicalOperand {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            operand,
            operator,
            literal,
        } = self;
        format!("{operand} operand of `{operator}` is the constant `{literal}`")
    }
}

impl AstRule for ConstantLogicalOperand {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // The operator token is aliased to the canonical lower-case keyword, so
        // its kind matches every spelling, including a keyword split by a line
        // continuation. `.not.` has two children instead of three and is
        // rejected here on purpose; `.eqv.`/`.neqv.` too, because a constant
        // operand there flips the result instead of being redundant.
        let operator = node.child(1)?.kind().to_string();
        if !matches!(operator.as_str(), ".and." | ".or.") {
            return None;
        }

        let mut diagnostics = vec![];
        for (index, operand) in [(0, "Left"), (2, "Right")] {
            let Some(side) = node.child(index) else {
                continue;
            };
            // `x .and. (.false.)` is the same expression as `x .and. .false.`.
            let side = strip_parentheses(side);
            if side.kind_id() != kind!("boolean_literal") {
                continue;
            }
            let Some(literal) = boolean_literal_text(&side) else {
                continue;
            };
            diagnostics.push(context.create_diagnostic(
                ConstantLogicalOperand {
                    operand: operand.to_string(),
                    operator: operator.clone(),
                    literal,
                },
                side,
            ));
        }

        if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["logical_expression"]
    }
}

/// The `.true.`/`.false.` text of a `boolean_literal`, without the optional
/// kind suffix (`.true._4`) and without the original spelling's case.
fn boolean_literal_text(node: &Node) -> Option<String> {
    let token = node.child(0)?.kind().to_lowercase();
    matches!(token.as_str(), ".true." | ".false.").then_some(token)
}

/// Unwrap `((...))` around an expression.
fn strip_parentheses(mut node: Node) -> Node {
    while node.kind_id() == kind!("parenthesized_expression") {
        let Some(inner) = node.child(1) else {
            break;
        };
        node = inner;
    }
    node
}
