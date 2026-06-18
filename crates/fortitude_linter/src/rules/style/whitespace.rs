/// Defines rules that enforce widely accepted whitespace rules.
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use crate::line_width::IndentWidth;
use fortitude_macros::ViolationMetadata;
use itertools::Itertools;
use ruff_macros::derive_message_formats;
use ruff_source_file::UniversalNewlines;
use ruff_text_size::{TextLen, TextRange, TextSize};
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};

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
    pub fn check(context: &CheckContext) -> Vec<Diagnostic> {
        let mut violations = Vec::new();
        for line in context.source_text().universal_newlines() {
            let whitespace_bytes: TextSize = line
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .map(TextLen::text_len)
                .sum();
            if whitespace_bytes > 0.into() {
                let range = TextRange::new(line.end() - whitespace_bytes, line.end());
                let edit = Edit::range_deletion(range);
                violations.push(
                    context
                        .create_diagnostic(Self {}, range)
                        .with_fix(Fix::safe_edit(edit)),
                );
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
    pub fn check(context: &CheckContext) -> Option<Diagnostic> {
        let text = context.source_text();

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
            let diagnostic = context
                .create_diagnostic(Self {}, range)
                .with_fix(Fix::safe_edit(edit));
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let source = context.source_file().to_source_code();
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
            // Unwraps are fine here because we're guaranteed to be at least 2
            // characters into the file, and `whitespace` is at most 1
            let span_start = comment_start
                .checked_sub(TextSize::try_from(whitespace).unwrap())
                .unwrap();

            let span = TextRange::new(span_start, comment_start);
            return some_vec!(
                context
                    .create_diagnostic(Self {}, span)
                    .with_fix(Fix::safe_edit(edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["comment"]
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let double_colon_start = node.start_byte();
        let double_colon_end = node.end_byte();

        let bytes = context.source_text().as_bytes();
        let has_space_before =
            double_colon_start > 0 && bytes[double_colon_start - 1].is_ascii_whitespace();
        let has_space_after =
            double_colon_end < bytes.len() && bytes[double_colon_end].is_ascii_whitespace();
        let before_edit = Edit::insertion(" ".to_string(), node.start_textsize());
        let after_edit = Edit::insertion(" ".to_string(), node.end_textsize());

        if !has_space_before {
            if !has_space_after {
                return some_vec!(
                    context
                        .create_diagnostic(Self {}, node)
                        .with_fix(Fix::safe_edits(before_edit, [after_edit]))
                );
            }
            return some_vec!(
                context
                    .create_diagnostic(Self {}, node)
                    .with_fix(Fix::safe_edit(before_edit))
            );
        } else if !has_space_after {
            return some_vec!(
                context
                    .create_diagnostic(Self {}, node)
                    .with_fix(Fix::safe_edit(after_edit))
            );
        }
        None
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["::" | kw]
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let node_as_str = node.to_text(context.source_text())?;

        let source = context.source_file().to_source_code();
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
            context
                .create_diagnostic(Self { is_open_bracket }, whitespace_range)
                .with_fix(Fix::safe_edit(Edit::range_deletion(whitespace_range)))
        )
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["(" | kw, "[" | kw, ")" | kw, "]" | kw]
    }
}

/// ## What it does
/// Checks that the correct indentation has been used
///
/// The complexity of handling semicolons requires that this
/// rule removes any semicolons used midway through a line
///
/// ## Why is this bad?
/// Inconsistent indentation makes Fortran less readable and difficult to
/// understand the scoping of logic.
///
/// ## Options
/// - `check.incorrect-indent.indent-width`
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectIndent;

impl AlwaysFixableViolation for IncorrectIndent {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Invalid indentation".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace with correct spaces".to_string()
    }
}

pub(crate) fn check_incorrect_indent(context: &CheckContext, root: &Node) -> Vec<Diagnostic> {
    let mut violations = Vec::new();

    let indent_width = context.settings().indent_width;
    let mut next_expected_indent = 0;
    let mut in_line_continuation = false;

    const BEGIN_SCOPE_NODES: &[&str] = &[
        "program_statement",
        "module_statement",
        "subroutine_statement",
        "function_statement",
        "derived_type_statement",
        "block_construct",
        "block_label_start_expression",
        "if_statement",
    ];
    const ZERO_INDENT_NODES: &[&str] = &[
        "preproc_if",
        "preproc_ifdef",
        "preproc_elifdef",
        "preproc_else",
        "preproc_include",
        "preproc_def",
        "preproc_function_def",
    ];
    const SCOPED_ZERO_INDENT_NODES: &[&str] = &["contains_statement"];
    const END_SCOPE_NODES: &[&str] = &[
        "end_program_statement",
        "end_module_statement",
        "end_subroutine_statement",
        "end_function_statement",
        "end_type_statement",
        "end_block_construct_statement",
        "end_if_statement",
    ];

    for line in context.source_text().universal_newlines() {
        // Skip empty lines and lines with only whitespace
        if line.trim().is_empty() {
            continue;
        }

        // Get current indent for line
        let line_indent = line.chars().take_while(|c| [' ', '\t'].contains(c)).count();

        // Loop through line until all semicolons have been accounted for
        let mut line_segment_start = line.start();
        let mut line_segment_end = line_segment_start;
        let mut is_first_segment = true;
        let mut edit_string: String = "".to_string();
        let line_contains_semicolon = line.contains(';');
        for line_segment in line.split_inclusive(';') {
            // Get the range which defines the location of the previous semicolon plus whitespace
            line_segment_start = line_segment_end;
            line_segment_end = line_segment_end + TextSize::from(line_segment.len() as u32);

            // Count leading spaces
            let leading_spaces = line_segment
                .chars()
                .take_while(|c| [' ', '\t'].contains(c))
                .count();

            // Get the first none whitespace node
            let content_start =
                line_segment_start + TextSize::try_from(leading_spaces as u32).unwrap();

            // Determine what the indentation should be for the next line using the first node for this line
            let mut current_expected_indent = next_expected_indent;
            if let Some(line_segment_node) = root
                .named_descendant_for_byte_range(content_start.to_usize(), content_start.to_usize())
            {
                let node_kind = &line_segment_node.kind();

                // Determine expected indent bases on tree-sitter node kind
                if BEGIN_SCOPE_NODES.contains(node_kind) {
                    next_expected_indent = current_expected_indent + indent_width;
                } else if END_SCOPE_NODES.contains(node_kind) {
                    if next_expected_indent < indent_width {
                        current_expected_indent = 0;
                        next_expected_indent = 0;
                    } else {
                        current_expected_indent = current_expected_indent - indent_width;
                        next_expected_indent = current_expected_indent;
                    }
                } else if ZERO_INDENT_NODES.contains(node_kind) {
                    current_expected_indent = 0;
                } else if SCOPED_ZERO_INDENT_NODES.contains(node_kind) {
                    current_expected_indent = current_expected_indent - indent_width;
                } else {
                    // Determine indent change based on line continuation char "&"
                    if !in_line_continuation && line.trim().ends_with("&") {
                        next_expected_indent = current_expected_indent + indent_width;
                        in_line_continuation = true;
                    } else if in_line_continuation && !line.trim().ends_with("&") {
                        next_expected_indent = current_expected_indent - indent_width;
                        in_line_continuation = false;
                    }
                }
            }

            // Include previous semicolon if present
            line_segment_start = if (is_first_segment && line.starts_with(';')) || !is_first_segment
            {
                line_segment_start - TextSize::try_from(1usize).unwrap()
            } else {
                line_segment_start
            };

            // Compare with the expected number of leading spaces
            if leading_spaces != current_expected_indent || line_contains_semicolon {
                if current_expected_indent > 0 {
                    let new_indent = " ".repeat(current_expected_indent);
                    if is_first_segment {
                        edit_string =
                            format!("{}{}{}", edit_string, new_indent, line_segment.trim());
                    } else {
                        edit_string =
                            format!("{}\n{}{}", edit_string, new_indent, line_segment.trim());
                    }
                } else {
                    edit_string = format!("{}{}", edit_string, line_segment.trim());
                };
                // Remove semicolons
                edit_string = edit_string.chars().filter(|c| *c != ';').join("");
            }

            if is_first_segment {
                is_first_segment = false;
            }
        }

        if !edit_string.is_empty() {
            let visual_end = if !line_contains_semicolon {
                if line_indent > 0 {
                    line_segment_start + TextSize::try_from(line_indent).unwrap()
                } else {
                    line_segment_start + TextSize::try_from(1usize).unwrap()
                }
            } else {
                line.end()
            };

            violations.push(
                context
                    .create_diagnostic(IncorrectIndent, TextRange::new(line.start(), visual_end))
                    .with_fix(Fix::safe_edit(Edit::range_replacement(
                        edit_string,
                        TextRange::new(line.start(), line.end()),
                    ))),
            );
        }
    }

    violations
}
