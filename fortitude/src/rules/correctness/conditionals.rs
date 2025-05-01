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

/// Given an if statement, return true if it is inline.
pub fn inline_if_statement(node: &Node) -> bool {
    node.child(2).is_none_or(|n| n.kind() != "then")
}
