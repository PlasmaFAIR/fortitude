use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
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
pub(crate) struct IfStatementSemicolon {}

impl AlwaysFixableViolation for IfStatementSemicolon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Semicolon following inline if-statement is misleading".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace with newline, or convert to `if(...) then` statement".to_string()
    }
}
impl AstRule for IfStatementSemicolon {
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
            return some_vec!(
                Diagnostic::from_node(IfStatementSemicolon {}, &semicolon_node)
                    .with_fix(Fix::safe_edit(edit))
            );
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

        // Check if the if statement ends on a different line than it starts.
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        if end_line > start_line {
            let content = ifthenify(node, src)?;
            let edit = node.edit_replacement(src, content);
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
    let source_text = src.source_text();
    // check that the node is an if statement without an end if statement
    if !inline_if_statement(node) {
        return None;
    }
    // Divide the if statement into everything up to the end of the condition
    // and the body.
    let condition_node = node.child_with_name("parenthesized_expression")?;
    let condition_end_byte = condition_node.end_byte();
    let mut body_start_byte = condition_end_byte;
    let source_bytes = source_text.as_bytes();
    // Skip over any whitespace or line continuations between the condition
    // and the body.
    while body_start_byte < node.end_byte()
        && (source_bytes[body_start_byte].is_ascii_whitespace()
            || source_bytes[body_start_byte] == b'&')
    {
        body_start_byte += 1;
    }
    // Concatenate bytes to the end of the condition, " then\n", the body, and
    // "\nend if". Take care to get the indentation right.
    let if_start_byte = node.start_byte();
    let if_end_byte = node.end_byte();
    let indentation = node.indentation(src);
    let mut correction = String::new();
    correction.push_str(&source_text[if_start_byte..condition_end_byte]);
    correction.push_str(" then\n");
    correction.push_str(&indentation);
    correction.push_str("  "); // Assume two-space indent for the if body
    correction.push_str(&source_text[body_start_byte..if_end_byte]);
    correction.push('\n');
    correction.push_str(&indentation);
    correction.push_str("end if");
    Some(correction)
}
