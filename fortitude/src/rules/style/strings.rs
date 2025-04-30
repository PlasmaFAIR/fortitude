use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};

/// ## What does it do?
/// Catches use of single-quoted strings.
///
/// ## Why is this bad?
/// For consistency, all strings should be either single-quoted or double-quoted.
/// Here, we enforce the use of double-quoted strings as this is the most common
/// style in other languages.
///
/// An exception is made for single-quoted strings that contain a `'"'` character,
/// as this is the preferred way to include double quotes in a string.
///
/// Fixes are not available for single-quoted strings containing escaped single
/// quotes (`"''"`).
#[derive(ViolationMetadata)]
pub(crate) struct SingleQuoteString {
    contains_escaped_quotes: bool,
}

impl Violation for SingleQuoteString {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Character string uses single quotes".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        if self.contains_escaped_quotes {
            return None;
        }
        Some("Replace with '\"'".to_string())
    }
}

impl AstRule for SingleQuoteString {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let text = node.to_text(src.source_text())?;
        if text.starts_with("'") && text.ends_with("'") && !text.contains('"') {
            // Search for occurrence of escaped single quotes within the string.
            // These are double single quotes, e.g. "''"
            if text.contains("''") && text.len() > 2 {
                return Some(vec![Diagnostic::from_node(
                    Self {
                        contains_escaped_quotes: true,
                    },
                    node,
                )]);
            }

            let start_byte = node.start_textsize();
            let end_byte = node.end_textsize();
            let edit_start =
                Edit::replacement("\"".to_string(), start_byte, start_byte + TextSize::from(1));
            let edit_end =
                Edit::replacement("\"".to_string(), end_byte - TextSize::from(1), end_byte);
            return some_vec!(Diagnostic::from_node(
                Self {
                    contains_escaped_quotes: false,
                },
                node,
            )
            .with_fix(Fix::safe_edits(edit_start, [edit_end])));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["string_literal"]
    }
}
