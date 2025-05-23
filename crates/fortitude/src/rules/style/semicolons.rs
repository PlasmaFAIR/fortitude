use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};

fn semicolon_is_superfluous(node: &Node) -> bool {
    let line_number = node.start_position().row;
    let prev_node = node.prev_sibling();
    let next_node = node.next_sibling();
    // Test it has at least one sibling.
    if prev_node.is_none() && next_node.is_none() {
        return true;
    }
    // Test it is at beginning of a line. If the previous sibling is on an earlier line,
    // or if there is no previous sibling, then it is at the beginning of a line.
    // Also check that the previous sibling isn't a semicolon!
    if let Some(prev_node) = prev_node {
        let prev_line_number = prev_node.start_position().row;
        if prev_line_number < line_number {
            return true;
        }
        if prev_line_number == line_number && prev_node.kind() == ";" {
            return true;
        }
    } else {
        return true;
    }
    // Test it is at the end of a line. If the next sibling is on a later line, or if
    // there is no next sibling, then it is at the end of a line. Also check that the
    // next sibling isn't a comment or another semicolon!
    if let Some(next_node) = next_node {
        let next_line_number = next_node.start_position().row;
        if next_line_number > line_number {
            return true;
        }
        if next_line_number == line_number && next_node.kind() == "comment" {
            return true;
        }
    } else {
        return true;
    }
    false
}

/// ## What does it do?
/// Catches a semicolon at the end of a line of code.
///
/// ## Why is this bad?
/// Many languages use semicolons to denote the end of a statement, but in Fortran each
/// line of code is considered its own statement (unless it ends with a line
/// continuation character, `'&'`). Semicolons may be used to separate multiple
/// statements written on the same line, but a semicolon at the end of a line has no
/// effect.
///
/// A semicolon at the beginning of a statement similarly has no effect, nor do
/// multiple semicolons in sequence.
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousSemicolon {}

impl AlwaysFixableViolation for SuperfluousSemicolon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "unnecessary semicolon".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove this character".to_string()
    }
}

impl AstRule for SuperfluousSemicolon {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if semicolon_is_superfluous(node) {
            let edit = node.edit_delete(src);
            return some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(edit)));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![";"]
    }
}

/// ## What does it do?
/// Catches multiple statements on the same line separated by a semicolon.
///
/// ## Why is this bad?
/// This can have a detrimental effect on code readability.
#[derive(ViolationMetadata)]
pub(crate) struct MultipleStatementsPerLine {}

impl AlwaysFixableViolation for MultipleStatementsPerLine {
    #[derive_message_formats]
    fn message(&self) -> String {
        "multiple statements per line".to_string()
    }

    fn fix_title(&self) -> String {
        "Separate over two lines".to_string()
    }
}

impl AstRule for MultipleStatementsPerLine {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if semicolon_is_superfluous(node) {
            return None;
        }
        let indentation = node.indentation(src);
        let start = node.start_byte();
        let mut end = node.end_byte();
        let text = src.source_text().as_bytes();
        while text[end] == b' ' || text[end] == b'\t' {
            end += 1;
        }
        let edit = Edit::replacement(
            format!("\n{indentation}"),
            TextSize::try_from(start).unwrap(),
            TextSize::try_from(end).unwrap(),
        );

        some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![";"]
    }
}
