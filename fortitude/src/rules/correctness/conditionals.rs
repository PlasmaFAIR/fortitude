use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::{find_newline, LineEnding, SourceFile};
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What does it do?
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
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // If this is an `if (...) then` construct, exit early
        if !inline_if_statement(node) {
            return None;
        }

        // Check that the if statement is followed directly by a semicolon
        if let Some(semicolon_node) = node.next_sibling().filter(|n| n.kind() == ";") {
            // Find the first following non-semicolon node, and check that it's
            // on the same line as the first semicolon node.
            // Comments on the same line are permitted.
            let mut current_node = semicolon_node;
            while let Some(next) = current_node.next_sibling() {
                if next.kind() == ";" {
                    current_node = next;
                    continue;
                }
                if next.start_position().row != semicolon_node.start_position().row
                    || next.kind() == "comment"
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
            let text = src.source_text().as_bytes();
            while text[end] == b' ' || text[end] == b'\t' {
                end += 1;
            }
            let end = TextSize::try_from(end).unwrap();
            let indentation = node.indentation(src);
            let edit = Edit::replacement(format!("\n{indentation}"), start, end);
            return some_vec!(Diagnostic::from_node(
                MisleadingInlineIfSemicolon {},
                &semicolon_node
            )
            .with_fix(Fix::safe_edit(edit)));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["if_statement"]
    }
}

/// ## What does it do?
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
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // If this is an `if (...) then` construct, exit early
        if !inline_if_statement(node) {
            return None;
        }

        // Check if the condition is immediately following by a continuation
        let body_start = node
            .child_with_name("parenthesized_expression")?
            .next_sibling()?;
        if body_start.kind() == "&" {
            let content = ifthenify(node, src)?;
            let start_byte = node.start_textsize();
            let end_byte = node.end_textsize();
            let edit = Edit::replacement(content, start_byte, end_byte);
            return some_vec!(
                Diagnostic::from_node(MisleadingInlineIfContinuation {}, node)
                    .with_fix(Fix::safe_edit(edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["if_statement"]
    }
}

/// Given an if statement, return true if it is inline.
pub fn inline_if_statement(node: &Node) -> bool {
    node.child_with_name("end_if_statement").is_none()
}

/// Given an if statement node, convert it from a one-line if statement
/// to a block if statement.
/// Returns None if the node is already a block if statement.
pub fn ifthenify(node: &Node, src: &SourceFile) -> Option<String> {
    // check that the node is an if statement without an end if statement
    if !inline_if_statement(node) {
        return None;
    }
    let text = node.to_text(src.source_text())?.trim();
    let bytes = text.as_bytes();
    // If CrLf line endings are used, be careful to keep them in the fix
    let nl = find_newline(text)
        .map(|(_, ending)| ending)
        .unwrap_or(LineEnding::Lf)
        .as_str();
    // Divide the if statement into everything up to the end of the condition
    // and the body.
    let condition_node = node.child_with_name("parenthesized_expression")?;
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
    let indentation = node.indentation(src);
    // Assume two-space indent for the if body
    Some(format!(
        "{} then{nl}{indentation}  {}{nl}{indentation}end if",
        prelude, body
    ))
}
