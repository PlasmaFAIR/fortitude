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
/// if (condition) then
///    print *, "Hello"
/// end if
/// print *, "World"
/// ```
///
/// When applying fixes, the if statement is converted to the second form and
/// the semicolon is removed.
#[derive(ViolationMetadata)]
pub(crate) struct IfStatementSemicolon {}

impl AlwaysFixableViolation for IfStatementSemicolon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Semicolon following inline if-statement is misleading".to_string()
    }

    fn fix_title(&self) -> String {
        "Convert to `if(...) then` statement".to_string()
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
            let if_edit = node.edit_replacement(src, ifthenify(node, src)?);
            let indentation = node.indentation(src);
            // To replace the semicolon node, we should also replace all
            // trailing whitespace.
            let semicolon_start = semicolon_node.start_textsize();
            let mut semicolon_end = semicolon_node.end_byte();
            let text = src.source_text().as_bytes();
            while text[semicolon_end] == b' ' || text[semicolon_end] == b'\t' {
                semicolon_end += 1;
            }
            let semicolon_end = TextSize::try_from(semicolon_end).unwrap();
            let semicolon_edit =
                Edit::replacement(format!("\n{indentation}"), semicolon_start, semicolon_end);
            return some_vec!(
                Diagnostic::from_node(IfStatementSemicolon {}, &semicolon_node)
                    .with_fix(Fix::safe_edits(if_edit, [semicolon_edit]))
            );
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["if_statement"]
    }
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
    correction.push_str(&source_text[condition_end_byte..if_end_byte]);
    correction.push('\n');
    correction.push_str(&indentation);
    correction.push_str("end if");
    Some(correction)
}

/// Given an if statement, return true if it is inline.
pub fn inline_if_statement(node: &Node) -> bool {
    node.child(2).is_none_or(|n| n.kind() != "then")
}
