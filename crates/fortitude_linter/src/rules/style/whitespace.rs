/// Defines rules that enforce widely accepted whitespace rules.
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::{SourceFile, UniversalNewlines};
use ruff_text_size::{TextLen, TextRange, TextSize};
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};

/// ## What does it do?
/// Checks for trailing whitespace.
///
/// ## Why is this bad?
/// Trailing whitespace is difficult to spot, and as some editors will remove it
/// automatically while others leave it, it can cause unwanted 'diff noise' in
/// shared projects.
#[derive(ViolationMetadata)]
pub(crate) struct TrailingWhitespace {}

impl AlwaysFixableViolation for TrailingWhitespace {
    #[derive_message_formats]
    fn message(&self) -> String {
        "trailing whitespace".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove trailing whitespace".to_string()
    }
}

impl TrailingWhitespace {
    pub fn check(source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let mut violations = Vec::new();
        for line in source.text().universal_newlines() {
            let whitespace_bytes: TextSize = line
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .map(TextLen::text_len)
                .sum();
            if whitespace_bytes > 0.into() {
                let range = TextRange::new(line.end() - whitespace_bytes, line.end());
                let edit = Edit::range_deletion(range);
                violations.push(Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit)));
            }
        }
        violations
    }
}

/// ## What does it do?
/// Checks for the absence of a new line at the end of the file.
///
/// ## Why is this bad?
/// POSIX standards state that a line is a sequence of characters
/// ending with a newline character. Some tools may not handle files
/// that do not end with a newline correctly, leading to potential issues
/// in file processing, version control diffs, and concatenation of files.
#[derive(ViolationMetadata)]
pub(crate) struct MissingNewlineAtEndOfFile {}

impl AlwaysFixableViolation for MissingNewlineAtEndOfFile {
    #[derive_message_formats]
    fn message(&self) -> String {
        "missing newline at end of file".to_string()
    }

    fn fix_title(&self) -> String {
        "Add newline at end of file".to_string()
    }
}

impl MissingNewlineAtEndOfFile {
    pub fn check(source_file: &SourceFile) -> Option<Diagnostic> {
        let source = source_file.to_source_code();
        let text = source.text();

        // Ignore empty and BOM only files.
        if text.is_empty() || text == "\u{feff}" {
            return None;
        }

        // Check that the last character is a newline
        if !text.ends_with(['\r', '\n']) {
            // Determine if the file is using Windows-style line endings
            let newline = if text.contains("\r\n") {
                "\r\n"
            } else if text.contains('\r') {
                "\r"
            } else {
                "\n"
            };
            let range = TextRange::empty(text.text_len());
            let edit = Edit::insertion(newline.to_string(), range.start());
            let diagnostic = Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit));
            Some(diagnostic)
        } else {
            None
        }
    }
}

/// ## What does it do?
/// Checks for inline comments that aren't preceded by at least two spaces.
///
/// ## Why is this bad?
/// Inline comments that aren't separated from code by any whitespace can make
/// code hard to read. Other language style guides recommend the use of two
/// spaces before inline comments, so we recommend the same here.
///
/// ## References
/// - [PEP8 Python Style Guide](https://peps.python.org/pep-0008/)
/// - [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html#Horizontal_Whitespace)
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectSpaceBeforeComment {}

impl AlwaysFixableViolation for IncorrectSpaceBeforeComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        "need at least 2 spaces before inline comment".to_string()
    }

    fn fix_title(&self) -> String {
        "Add extra whitespace".to_string()
    }
}
impl AstRule for IncorrectSpaceBeforeComment {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let source = src.to_source_code();
        let comment_start = node.start_textsize();
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
            let edit = Edit::insertion("  "[whitespace..].to_string(), comment_start);
            return some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(edit)));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["comment"]
    }
}

/// ## What does it do?
/// Checks for `::` that aren't surrounded by a space on either side.
///
/// ## Why is this bad?
/// Omitting any whitespace surrounding the double colon separator can make code harder
/// to read:
///
/// ```f90
/// character(len=256)::x
/// ! vs
/// character(len=256) :: x
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectSpaceAroundDoubleColon {}

impl AlwaysFixableViolation for IncorrectSpaceAroundDoubleColon {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing space around `::`".to_string()
    }

    fn fix_title(&self) -> String {
        "Add extra whitespace".to_string()
    }
}
impl AstRule for IncorrectSpaceAroundDoubleColon {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let double_colon_start = node.start_byte();
        let double_colon_end = node.end_byte();

        let bytes = src.source_text().as_bytes();
        let has_space_before =
            double_colon_start > 0 && bytes[double_colon_start - 1].is_ascii_whitespace();
        let has_space_after =
            double_colon_end < bytes.len() && bytes[double_colon_end].is_ascii_whitespace();
        let before_edit = Edit::insertion(" ".to_string(), node.start_textsize());
        let after_edit = Edit::insertion(" ".to_string(), node.end_textsize());

        if !has_space_before {
            if !has_space_after {
                return some_vec!(
                    Diagnostic::from_node(Self {}, node)
                        .with_fix(Fix::safe_edits(before_edit, [after_edit]))
                );
            }
            return some_vec!(
                Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(before_edit))
            );
        } else if !has_space_after {
            return some_vec!(
                Diagnostic::from_node(Self {}, node).with_fix(Fix::safe_edit(after_edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["::"]
    }
}

/// ## What does it do?
/// Checks for spaces between brackets and their contents.
///
/// ## Why is this bad?
/// Including spaces between brackets and their contents can lead to
/// inconsistent formatting and readability issues if the same style is
/// not applied consistently throughout the codebase.
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectSpaceBetweenBrackets {
    is_open_bracket: bool,
}

impl AlwaysFixableViolation for IncorrectSpaceBetweenBrackets {
    #[derive_message_formats]
    fn message(&self) -> String {
        if self.is_open_bracket {
            "Should be 0 space after the opening bracket".to_string()
        } else {
            "Should be 0 space before the closing bracket".to_string()
        }
    }

    fn fix_title(&self) -> String {
        "remove extra whitespace".to_string()
    }
}
impl AstRule for IncorrectSpaceBetweenBrackets {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let node_as_str = node.to_text(src.source_text())?;

        let source = src.to_source_code();
        let bracket_start = node.start_textsize();
        let bracket_end = node.end_textsize();
        let line_index = source.line_index(bracket_end);

        let is_open_bracket = matches!(node_as_str, "(" | "[");
        let (whitespace_start, whitespace_end) = if is_open_bracket {
            // Get line after bracket
            let line_end = source.line_end(line_index);
            let range_after = TextRange::new(bracket_end, line_end);
            let line_after = source.slice(range_after);

            // Ignore if preceding a line wrap, i.e. &
            if line_after.trim_start().starts_with('&') {
                return None;
            }

            // Count whitespace characters after the bracket
            let whitespace_iter = line_after.chars().take_while(|c| c.is_whitespace());
            let whitespace_count = whitespace_iter.count();
            let whitespace_end = bracket_end + TextSize::from(whitespace_count as u32);

            (bracket_end, whitespace_end)
        } else {
            // Get line before bracket
            let line_start = source.line_start(line_index);
            let range_before = TextRange::new(line_start, bracket_start);
            let line_before = source.slice(range_before);

            // Ignore if following a line wrap, i.e. &
            if line_before.trim_end().ends_with('&') || line_before.trim().is_empty() {
                return None;
            }

            // Count whitespace characters before the bracket
            let whitespace_iter = line_before.chars().rev().take_while(|c| c.is_whitespace());
            let whitespace_count = whitespace_iter.count();
            let whitespace_start = bracket_start - TextSize::from(whitespace_count as u32);

            (whitespace_start, bracket_start)
        };

        if whitespace_start == whitespace_end {
            return None; // No whitespace to fix
        }
        let whitespace_range = TextRange::new(whitespace_start, whitespace_end);

        // If the space is between empty brackets only raise for closing bracket
        let after = source.after(whitespace_end);
        if is_open_bracket && (after.starts_with(")") || after.starts_with("]")) {
            return None;
        }

        some_vec!(
            Diagnostic::new(Self { is_open_bracket }, whitespace_range)
                .with_fix(Fix::safe_edit(Edit::range_deletion(whitespace_range)))
        )
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["(", "[", ")", "]"]
    }
}
