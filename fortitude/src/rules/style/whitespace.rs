/// Defines rules that enforce widely accepted whitespace rules.
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::{TextLen, TextRange, TextSize};
use tree_sitter::Node;

use crate::settings::Settings;
use crate::{ast::FortitudeNode, AstRule, FromAstNode, TextRule};

/// ## What does it do?
/// Checks for tailing whitespace
///
/// ## Why is this bad?
/// Trailing whitespace is difficult to spot, and as some editors will remove it
/// automatically while others leave it, it can cause unwanted 'diff noise' in
/// shared projects.
#[violation]
pub struct TrailingWhitespace {}

impl AlwaysFixableViolation for TrailingWhitespace {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("trailing whitespace")
    }

    fn fix_title(&self) -> String {
        format!("Remove trailing whitespace")
    }
}

impl TextRule for TrailingWhitespace {
    fn check(_settings: &Settings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let mut violations = Vec::new();
        for (idx, line) in source.text().lines().enumerate() {
            let whitespace_bytes: TextSize = line
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .map(TextLen::text_len)
                .sum();
            if whitespace_bytes > 0.into() {
                let line_end_byte = source.line_end_exclusive(OneIndexed::from_zero_indexed(idx));
                let range = TextRange::new(line_end_byte - whitespace_bytes, line_end_byte);
                let edit = Edit::range_deletion(range);
                violations.push(Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit)));
            }
        }
        violations
    }
}

/// ## What does it do?
/// Checks for inline comments that aren't preceeded by at least two spaces.
///
/// ## Why is this bad?
/// Inline comments that aren't separated from code by any whitespace can make
/// code hard to read. Other language style guides recommend the use of two
/// spaces before inline comments, so we recommend the same here.
///
/// ## References
/// - [PEP8 Python Style Guide](https://peps.python.org/pep-0008/)
/// - [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html#Horizontal_Whitespace)
#[violation]
pub struct IncorrectSpaceBeforeComment {}

impl AlwaysFixableViolation for IncorrectSpaceBeforeComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("need at least 2 spaces before inline comment")
    }

    fn fix_title(&self) -> String {
        format!("add extra whitespace")
    }
}
impl AstRule for IncorrectSpaceBeforeComment {
    fn check(
        _settings: &Settings,
        node: &Node,
        source_file: &SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let source = source_file.to_source_code();
        let comment_start = TextSize::try_from(node.start_byte()).unwrap();
        // Get the line up to the start of the comment
        let line_index = source.line_index(comment_start);
        let line_start = source.line_start(line_index);
        let range = TextRange::new(line_start, comment_start);
        let line = source.slice(range);
        // Count whitespace characters at the end of the line
        let whitespace = line.chars().rev().take_while(|c| c.is_whitespace()).count();
        // If the line is empty or just filled with whitespace, exit
        if line.len() == whitespace {
            return None;
        }
        if whitespace < 2 {
            let replacement = format!("{}{}", &"  "[whitespace..], node.to_text(source.text())?);
            let edit = node.edit_replacement(replacement);
            return some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(edit)));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["comment"]
    }
}
