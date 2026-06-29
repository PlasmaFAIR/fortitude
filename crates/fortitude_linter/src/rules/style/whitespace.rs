/// Defines rules that enforce widely accepted whitespace rules.
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use crate::rules::Rule;
use fortitude_macros::ViolationMetadata;
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
/// - `check.indent-width`
/// - `check.invalid-indentation-multiple.num-indents-for-associate-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-block-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-derived-type-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-do-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-function-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-if-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-interface-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-module-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-program-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-select-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-submodule-contents`
/// - `check.invalid-indentation-multiple.num-indents-for-line-continuation`
/// - `check.invalid-indentation-multiple.num-indents-for-subroutine-contents`
/// - `check.invalid-indentation-multiple.should-indent-associate-contents`
/// - `check.invalid-indentation-multiple.should-indent-block-contents`
/// - `check.invalid-indentation-multiple.should-indent-derived-type-contents`
/// - `check.invalid-indentation-multiple.should-indent-do-contents`
/// - `check.invalid-indentation-multiple.should-indent-function-contents`
/// - `check.invalid-indentation-multiple.should-indent-if-contents`
/// - `check.invalid-indentation-multiple.should-indent-interface-contents`
/// - `check.invalid-indentation-multiple.should-indent-module-contents`
/// - `check.invalid-indentation-multiple.should-indent-program-contents`
/// - `check.invalid-indentation-multiple.should-indent-select-contents`
/// - `check.invalid-indentation-multiple.should-indent-submodule-contents`
/// - `check.invalid-indentation-multiple.should-indent-subroutine-contents`
/// - `check.invalid-indentation-multiple.should-indent-after-line-continuation`
#[derive(ViolationMetadata)]
pub(crate) struct InvalidIndentationMultiple;

impl AlwaysFixableViolation for InvalidIndentationMultiple {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Invalid indentation".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace with correct spaces".to_string()
    }
}

/// ## What it does
/// Checks that preprocessor statements have zero indentation before the '#'
///
/// ## Why is this bad?
/// Preprocessor statements with indentation are invalid fortran
#[derive(ViolationMetadata)]
pub(crate) struct InvalidPreprocIndentation;

impl AlwaysFixableViolation for InvalidPreprocIndentation {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Preprocessor statements should have zero indentation".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove indentation".to_string()
    }
}

const BEGIN_SCOPE_NODES: [&str; 15] = [
    "program_statement",
    "module_statement",
    "submodule_statement",
    "subroutine_statement",
    "function_statement",
    "function",
    "derived_type_statement",
    "block_construct",
    "if_statement",
    "interface_statement",
    "procedure_qualifier",
    "select_case_statement",
    // loop and statement needed to catch case of checking parent of block_label_start_expression
    "do_loop",
    "do_statement",
    "associate_statement",
];
const PREPROC_NODES: [&str; 7] = [
    "preproc_if",
    "preproc_ifdef",
    "preproc_elifdef",
    "preproc_else",
    "preproc_include",
    "preproc_def",
    "preproc_function_def",
];
const SCOPED_ZERO_INDENT_NODES: [&str; 2] = ["contains_statement", "case_statement"];
const END_SCOPE_NODES: [&str; 12] = [
    "end_program_statement",
    "end_module_statement",
    "end_submodule_statement",
    "end_subroutine_statement",
    "end_function_statement",
    "end_type_statement",
    "end_block_construct_statement",
    "end_if_statement",
    "end_interface_statement",
    "end_select_statement",
    "end_do_loop_statement",
    "end_associate_statement",
];

fn split_segments_outside_quotes(line: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut chars = line.char_indices();

    while let Some((idx, ch)) = chars.next() {
        if in_quotes {
            if ['\'', '"'].contains(&ch) {
                in_quotes = false;
            }
        } else if ch == ';' {
            segments.push(&line[start..idx + ch.len_utf8()]);
            start = idx + ch.len_utf8();
        } else if ['\'', '"'].contains(&ch) {
            in_quotes = true;
        }
    }

    if start < line.len() {
        segments.push(&line[start..]);
    }

    segments
}

pub(crate) fn check_incorrect_indent(context: &CheckContext, root: &Node) -> Vec<Diagnostic> {
    let mut violations = Vec::new();

    let indent_width = context.settings().indent_width;

    let constructs_to_indent_map = &context
        .settings()
        .invalid_indentation_multiple
        .construct_to_indent_map;

    // Array to track both the number of scopes we are inside and their respective indents
    let mut scope_indents: Vec<usize> = Vec::new();

    let mut in_line_continuation = false;
    for line in context.source_text().universal_newlines() {
        // Skip empty lines and lines with only whitespace
        if line.trim().is_empty() {
            continue;
        }

        // Get current indent for line
        let line_indent = line.chars().take_while(|c| [' ', '\t'].contains(c)).count();

        // Booleans to determine the rule that has been broken
        let mut is_preproc_violation = false;
        // boolean to track if a line should be updated based on the users selected rules
        let mut edit_is_activated = context.is_rule_enabled(Rule::InvalidIndentationMultiple);

        // Loop through line until all semicolons outside quoted strings have been accounted for
        let mut line_segment_start = line.start();
        let mut line_segment_end = line_segment_start;
        let mut is_first_segment = true;
        let mut edit_string: String = "".to_string();
        let line_segments = split_segments_outside_quotes(&line);
        let line_contains_semicolon = line_segments.iter().any(|segment| segment.ends_with(';'));
        for line_segment in line_segments {
            // Get the range which defines the location of the previous semicolon plus whitespace
            line_segment_start = line_segment_end;
            line_segment_end = line_segment_end + TextSize::from(line_segment.len() as u32);

            // Count leading spaces
            let leading_spaces = line_segment.chars().take_while(|c| *c == ' ').count()
                + indent_width * line_segment.chars().take_while(|c| *c == '\t').count();

            // Get the first none whitespace node
            let content_start =
                line_segment_start + TextSize::try_from(leading_spaces as u32).unwrap();

            // Boolean to track if this line segment continued onto the next line via a '&'
            let line_segment_has_continuation = line_segment.trim().ends_with('&');

            // Determine what the indentation should be for this line segment using the first node for this line and the current scope
            let mut current_expected_indent = *scope_indents.last().unwrap_or(&0usize);
            if let Some(line_segment_node) = root
                .named_descendant_for_byte_range(content_start.to_usize(), content_start.to_usize())
            {
                // Handle block labels, module procedures and functions beginning with their return type by taking their parent
                let node = if (matches!(line_segment_node.kind(), "block_label_start_expression"))
                    || (matches!(line_segment_node.kind(), "intrinsic_type")
                        && !line_segment.contains("::")
                        || (matches!(line_segment_node.kind(), "procedure_qualifier")))
                {
                    line_segment_node
                        .ancestors()
                        .next()
                        .unwrap_or(line_segment_node)
                } else {
                    line_segment_node.clone()
                };
                let node_kind = node.kind();

                // Determine expected indent bases on tree-sitter node kind
                if BEGIN_SCOPE_NODES.contains(&node_kind) && !node.inline_if_statement() {
                    if edit_is_activated {
                        scope_indents.push(
                            current_expected_indent
                                + indent_width * constructs_to_indent_map.get(node_kind).unwrap(),
                        );
                    } else {
                        scope_indents.push(leading_spaces);
                    }
                } else if END_SCOPE_NODES.contains(&node_kind) {
                    scope_indents.pop();
                    current_expected_indent = *scope_indents.last().unwrap_or(&0usize);
                } else if PREPROC_NODES.contains(&node_kind) {
                    edit_is_activated = edit_is_activated
                        || context.is_rule_enabled(Rule::InvalidPreprocIndentation);
                    is_preproc_violation = true;
                    current_expected_indent = 0usize;
                } else if SCOPED_ZERO_INDENT_NODES.contains(&node_kind) {
                    current_expected_indent = *scope_indents.iter().rev().nth(1).unwrap_or(&0usize);
                }

                // Determine indent change based on line continuation char "&"
                if edit_is_activated {
                    if !in_line_continuation && line_segment_has_continuation {
                        in_line_continuation = true;
                        scope_indents.push(
                            current_expected_indent
                                + indent_width
                                    * constructs_to_indent_map.get("line_continuation").unwrap(),
                        );
                    } else if in_line_continuation && !line_segment_has_continuation {
                        in_line_continuation = false;
                        scope_indents.pop();
                        // Align single closing brace with the outer indent
                        if [")", "]", "}", r"\)"].contains(&line_segment.trim()) {
                            current_expected_indent =
                                *scope_indents.iter().rev().nth(1).unwrap_or(&0usize);
                        }
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
                let new_indent = " ".repeat(current_expected_indent);
                if is_first_segment {
                    edit_string = format!("{}{}{}", edit_string, new_indent, line_segment.trim());
                } else {
                    edit_string = format!("{}\n{}{}", edit_string, new_indent, line_segment.trim());
                }
                // Remove semicolons that are not inside quotes
                let mut in_quotes = false;
                edit_string = edit_string
                    .chars()
                    .filter(|c| {
                        if ['\'', '"'].contains(c) {
                            in_quotes = !in_quotes;
                        }
                        !(matches!(c, ';') && !in_quotes)
                    })
                    .collect();
            }

            is_first_segment = false;
        }

        if !edit_string.is_empty() {
            let visual_end = if !line_contains_semicolon {
                line_segment_start + TextSize::try_from(std::cmp::max(line_indent, 1)).unwrap()
            } else {
                line.end()
            };

            let range = TextRange::new(line.start(), visual_end);
            let fix = Fix::safe_edit(Edit::range_replacement(
                edit_string,
                TextRange::new(line.start(), line.end()),
            ));

            if let Some(diagnostic) =
                context.create_diagnostic_if_enabled(InvalidIndentationMultiple, range)
            {
                violations.push(diagnostic.with_fix(fix));
            } else if is_preproc_violation {
                if let Some(diagnostic) =
                    context.create_diagnostic_if_enabled(InvalidPreprocIndentation, range)
                {
                    violations.push(diagnostic.with_fix(fix));
                };
            }
        }
    }

    violations
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::{collections::HashMap, fmt::Display};

    #[derive(Debug, Clone, CacheKey)]
    pub struct InvalidIndentationMultipleSettings {
        pub construct_to_indent_map: HashMap<String, usize>,
        pub should_indent_program_contents: bool,
        pub should_indent_module_contents: bool,
        pub should_indent_submodule_contents: bool,
        pub should_indent_subroutine_contents: bool,
        pub should_indent_function_contents: bool,
        pub should_indent_derived_type_contents: bool,
        pub should_indent_block_contents: bool,
        pub should_indent_if_contents: bool,
        pub should_indent_interface_contents: bool,
        pub should_indent_select_contents: bool,
        pub should_indent_do_contents: bool,
        pub should_indent_associate_contents: bool,
        pub should_indent_after_line_continuation: bool,
        pub num_indents_for_program_contents: usize,
        pub num_indents_for_module_contents: usize,
        pub num_indents_for_submodule_contents: usize,
        pub num_indents_for_subroutine_contents: usize,
        pub num_indents_for_function_contents: usize,
        pub num_indents_for_derived_type_contents: usize,
        pub num_indents_for_block_contents: usize,
        pub num_indents_for_if_contents: usize,
        pub num_indents_for_interface_contents: usize,
        pub num_indents_for_select_contents: usize,
        pub num_indents_for_do_contents: usize,
        pub num_indents_for_associate_contents: usize,
        pub num_indents_for_line_continuation: usize,
    }

    impl Default for InvalidIndentationMultipleSettings {
        fn default() -> Self {
            let construct_to_indent_map: HashMap<String, usize> = HashMap::new();
            let mut settings = Self {
                construct_to_indent_map,
                should_indent_program_contents: true,
                should_indent_module_contents: true,
                should_indent_submodule_contents: true,
                should_indent_subroutine_contents: true,
                should_indent_function_contents: true,
                should_indent_derived_type_contents: true,
                should_indent_block_contents: true,
                should_indent_if_contents: true,
                should_indent_interface_contents: true,
                should_indent_select_contents: true,
                should_indent_do_contents: true,
                should_indent_associate_contents: true,
                should_indent_after_line_continuation: true,
                num_indents_for_program_contents: 1usize,
                num_indents_for_module_contents: 1usize,
                num_indents_for_submodule_contents: 1usize,
                num_indents_for_subroutine_contents: 1usize,
                num_indents_for_function_contents: 1usize,
                num_indents_for_derived_type_contents: 1usize,
                num_indents_for_block_contents: 1usize,
                num_indents_for_if_contents: 1usize,
                num_indents_for_interface_contents: 1usize,
                num_indents_for_select_contents: 1usize,
                num_indents_for_do_contents: 1usize,
                num_indents_for_associate_contents: 1usize,
                num_indents_for_line_continuation: 1usize,
            };
            settings.populate_construct_to_indent_map()
        }
    }

    impl InvalidIndentationMultipleSettings {
        pub fn populate_construct_to_indent_map(&mut self) -> Self {
            self.construct_to_indent_map.insert(
                "program_statement".to_string(),
                if self.should_indent_program_contents {
                    self.num_indents_for_program_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "module_statement".to_string(),
                if self.should_indent_module_contents {
                    self.num_indents_for_module_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "submodule_statement".to_string(),
                if self.should_indent_submodule_contents {
                    self.num_indents_for_submodule_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "subroutine_statement".to_string(),
                if self.should_indent_subroutine_contents {
                    self.num_indents_for_subroutine_contents
                } else {
                    0usize
                },
            );

            let function_indent = if self.should_indent_function_contents {
                self.num_indents_for_function_contents
            } else {
                0usize
            };
            self.construct_to_indent_map
                .insert("function_statement".to_string(), function_indent);
            self.construct_to_indent_map
                .insert("function".to_string(), function_indent);

            self.construct_to_indent_map.insert(
                "derived_type_statement".to_string(),
                if self.should_indent_derived_type_contents {
                    self.num_indents_for_derived_type_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "block_construct".to_string(),
                if self.should_indent_block_contents {
                    self.num_indents_for_block_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "if_statement".to_string(),
                if self.should_indent_if_contents {
                    self.num_indents_for_if_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "interface_statement".to_string(),
                if self.should_indent_interface_contents {
                    self.num_indents_for_interface_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "select_case_statement".to_string(),
                if self.should_indent_select_contents {
                    self.num_indents_for_select_contents
                } else {
                    0usize
                },
            );

            let do_indent = if self.should_indent_do_contents {
                self.num_indents_for_do_contents
            } else {
                0usize
            };
            self.construct_to_indent_map
                .insert("do_loop".to_string(), do_indent);
            self.construct_to_indent_map
                .insert("do_statement".to_string(), do_indent);

            self.construct_to_indent_map.insert(
                "associate_statement".to_string(),
                if self.should_indent_associate_contents {
                    self.num_indents_for_associate_contents
                } else {
                    0usize
                },
            );

            self.construct_to_indent_map.insert(
                "line_continuation".to_string(),
                if self.should_indent_after_line_continuation {
                    self.num_indents_for_line_continuation
                } else {
                    0usize
                },
            );

            self.clone()
        }
    }

    impl Display for InvalidIndentationMultipleSettings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.invalid-indentation-multiple",
                fields = [
                    self.should_indent_program_contents,
                    self.should_indent_module_contents,
                    self.should_indent_submodule_contents,
                    self.should_indent_subroutine_contents,
                    self.should_indent_function_contents,
                    self.should_indent_derived_type_contents,
                    self.should_indent_block_contents,
                    self.should_indent_if_contents,
                    self.should_indent_interface_contents,
                    self.should_indent_select_contents,
                    self.should_indent_do_contents,
                    self.should_indent_associate_contents,
                    self.should_indent_after_line_continuation,
                    self.num_indents_for_program_contents,
                    self.num_indents_for_module_contents,
                    self.num_indents_for_submodule_contents,
                    self.num_indents_for_subroutine_contents,
                    self.num_indents_for_function_contents,
                    self.num_indents_for_derived_type_contents,
                    self.num_indents_for_block_contents,
                    self.num_indents_for_if_contents,
                    self.num_indents_for_interface_contents,
                    self.num_indents_for_select_contents,
                    self.num_indents_for_do_contents,
                    self.num_indents_for_associate_contents,
                    self.num_indents_for_line_continuation,
                ]
            }
            Ok(())
        }
    }
}
