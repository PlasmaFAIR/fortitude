/// Defines rules that enforce widely accepted whitespace rules.
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use crate::rules::Rule;
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;
use ruff_source_file::UniversalNewlines;
use ruff_text_size::{TextLen, TextRange, TextSize};

use crate::{AstRule, CheckContext, kind_ids};
use fortitude_sitter::traits::TextRanged;

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
        let node_as_str = node.text();

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
/// rule either removes any semicolons used midway through a line
/// or ignores any lines containing a semicolon. This logic can be
/// toggled using the `ignore-semicolons` option.
///
/// ## Why is this bad?
/// Inconsistent indentation makes Fortran less readable and difficult to
/// understand the scoping of logic.
///
/// ## Options
/// - `check.indent-width`
/// - `check.incorrect-indentation.ignore-semicolons`
/// - `check.incorrect-indentation.program-indent`
/// - `check.incorrect-indentation.module-indent`
/// - `check.incorrect-indentation.procedure-indent`
/// - `check.incorrect-indentation.derived-type-indent`
/// - `check.incorrect-indentation.control-flow-indent`
/// - `check.incorrect-indentation.interface-indent`
/// - `check.incorrect-indentation.line-continuation-indent`
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectIndentation {
    expected_indent: usize,
    semicolon_found: bool,
    label_found: bool,
}

impl AlwaysFixableViolation for IncorrectIndentation {
    #[derive_message_formats]
    fn message(&self) -> String {
        if self.semicolon_found {
            "Incorrect indentation and semicolon found".to_string()
        } else {
            "Incorrect indentation".to_string()
        }
    }

    fn fix_title(&self) -> String {
        if self.semicolon_found {
            "Remove semicolons and indent correctly".to_string()
        } else if self.label_found {
            "Replace with correct indentation".to_string()
        } else {
            format!(
                "Replace with the correct number of spaces, {}",
                self.expected_indent
            )
            .to_string()
        }
    }
}

/// ## What it does
/// Checks that preprocessor statements have zero indentation before the '#'
///
/// ## Why is this bad?
/// Preprocessor statements with indentation are invalid fortran
#[derive(ViolationMetadata)]
pub(crate) struct IndentedPreprocessorStatement;

impl AlwaysFixableViolation for IndentedPreprocessorStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Preprocessor statements should have zero indentation".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove indentation".to_string()
    }
}

const BEGIN_SCOPE_NODES: [&str; 18] = [
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
    "select_type_statement",
    "select_rank_statement",
    // loop and statement needed to catch case of checking parent of block_label_start_expression
    "do_loop",
    "do_statement",
    "associate_statement",
    "where_statement",
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
const SCOPED_ZERO_INDENT_NODES: [&str; 7] = [
    "contains_statement",
    "case_statement",
    "type_statement",
    "rank_statement",
    "else_clause",
    "elseif_clause",
    "elsewhere_clause",
];
const END_SCOPE_NODES: [&str; 13] = [
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
    "end_where_statement",
];

fn update_enclosing_quote(ch: &char, enclosing_quote: Option<char>) -> Option<char> {
    if ['\'', '"'].contains(ch) {
        if let Some(quote) = enclosing_quote {
            if quote == *ch {
                return None;
            }
        } else {
            return Some(*ch);
        }
    }
    enclosing_quote
}

fn split_segments_outside_quotes(line: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let chars = line.char_indices();

    let mut enclosing_quote: Option<char> = None;
    for (idx, ch) in chars {
        enclosing_quote = update_enclosing_quote(&ch, enclosing_quote);
        if enclosing_quote.is_none() && ch == ';' {
            segments.push(&line[start..idx + ch.len_utf8()]);
            start = idx + ch.len_utf8();
        }
    }

    if start < line.len() {
        segments.push(&line[start..]);
    }

    segments
}

pub(crate) fn check_incorrect_indent(context: &CheckContext, root: &Node) -> Vec<Diagnostic> {
    let mut violations = Vec::new();

    // Store settings
    let indent_width = context.settings().indent_width;
    let ignore_semicolons = context.settings().incorrect_indentation.ignore_semicolons;
    let constructs_to_indent_map = &context.settings().incorrect_indentation.indent_map();

    // Array to track both the number of scopes we are inside and their respective indents
    let mut scope_indents: Vec<usize> = Vec::new();

    let mut in_line_continuation = false;
    for line in context.source_text().universal_newlines() {
        // Skip empty lines and lines with only whitespace
        if line.trim().is_empty() {
            continue;
        }

        // Boolean to determine the rule that has been broken
        let mut is_preproc_violation = false;
        // boolean to track if a line should be updated based on the users selected rules
        let mut edit_is_activated = context.is_rule_enabled(Rule::IncorrectIndentation);
        // Get current indent for line to be used displaying the text range to be updated to the user.
        let mut visual_region_length = line.chars().take_while(|c| [' ', '\t'].contains(c)).count();

        // Loop through line until all semicolons outside quoted strings have been accounted for
        let mut line_segment_start = line.start();
        let mut line_segment_end = line_segment_start;
        let mut is_first_segment = true;
        let mut edit_string: String = "".to_string();
        let line_segments = split_segments_outside_quotes(&line);
        let line_contains_semicolon = line_segments.iter().any(|segment| segment.ends_with(';'));
        let mut line_has_label = false;
        for line_segment in line_segments {
            let mut label_text: String = "".to_string();
            // Get the range which defines the location of the previous semicolon plus whitespace
            line_segment_start = line_segment_end;
            line_segment_end += TextSize::from(line_segment.len() as u32);

            // Count leading spaces
            let leading_spaces = line_segment.chars().take_while(|c| *c == ' ').count()
                + indent_width.as_usize() * line_segment.chars().take_while(|c| *c == '\t').count();

            // Get the first none whitespace node
            let content_start = line_segment_start + TextSize::from(leading_spaces as u32);

            // Boolean to track if this line segment continued onto the next line via a '&'
            let line_segment_has_continuation = line_segment.trim().ends_with('&');

            // Determine what the indentation should be for this line segment using the first node for this line and the current scope
            let mut current_expected_indent = *scope_indents.last().unwrap_or(&0usize);
            if let Some(line_segment_node) = root.named_descendant_for_byte_range(
                content_start.to_usize(),
                content_start.to_usize() + 1,
            ) {
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
                    line_segment_node
                };

                // Handle statement_labels by taking their child
                if matches!(node.kind(), "statement_label") {
                    line_has_label = true;
                    label_text = context.source_text()[node.textrange()].to_string();
                    node.descendants().next().unwrap_or(node);
                };

                let node_kind = node.kind();

                // Determine expected indent based on tree-sitter node kind and line continuations '&'.
                // If we are in a line continuation, we just indent once.
                if in_line_continuation {
                    // If we are at the end of the line continuation, we remove the last indent for future lines.
                    if !line_segment_has_continuation {
                        in_line_continuation = false;
                        scope_indents.pop();
                        // Align single closing brace with the outer indent
                        if [")", "]", "}", r"\)"].contains(&line_segment.trim()) {
                            current_expected_indent =
                                *scope_indents.iter().rev().nth(1).unwrap_or(&0usize);
                        }
                    }
                // Otherwise, we indent based on the type of node which starts this line segment
                } else if BEGIN_SCOPE_NODES.contains(&node_kind) && !node.inline_if_statement() {
                    if edit_is_activated {
                        scope_indents.push(
                            current_expected_indent
                                + constructs_to_indent_map.get_indent(node_kind),
                        );
                    } else {
                        scope_indents.push(leading_spaces);
                    }
                } else if END_SCOPE_NODES.contains(&node_kind) {
                    scope_indents.pop();
                    current_expected_indent = *scope_indents.last().unwrap_or(&0usize);
                } else if PREPROC_NODES.contains(&node_kind) {
                    edit_is_activated =
                        context.is_rule_enabled(Rule::IndentedPreprocessorStatement);
                    is_preproc_violation = true;
                    current_expected_indent = 0usize;
                } else if SCOPED_ZERO_INDENT_NODES.contains(&node_kind) {
                    current_expected_indent = *scope_indents.iter().rev().nth(1).unwrap_or(&0usize);
                }

                // Account for entering a line continuation by adding one extra indent
                if !in_line_continuation && line_segment_has_continuation {
                    in_line_continuation = true;
                    scope_indents.push(
                        current_expected_indent
                            + constructs_to_indent_map.get_indent("line_continuation"),
                    );
                }
            }

            // Include previous semicolon if present
            line_segment_start = if !is_first_segment || line.starts_with(';') {
                line_segment_start - TextSize::new(1)
            } else {
                line_segment_start
            };

            // Populate the new replacement string if a violation has been found
            let indentation_mismatch = leading_spaces != current_expected_indent;
            #[allow(clippy::nonminimal_bool)] // Allow non-minimal bool to keep logic clear
            if (line_contains_semicolon && !ignore_semicolons)
                || (indentation_mismatch && (!line_contains_semicolon || !ignore_semicolons))
            {
                let mut new_line_segment = format!(
                    "{}{}",
                    " ".repeat(current_expected_indent),
                    line_segment.trim()
                );

                // Account for statement_labels
                if !label_text.is_empty() {
                    let mut new_indent_size = current_expected_indent;
                    let label_size = label_text.chars().count();
                    // If the label size matches the indent size add extra indent after label
                    new_indent_size = if new_indent_size <= label_size + 2 {
                        2
                    } else {
                        new_indent_size - label_size
                    };
                    let new_indent = format!("{}{}", label_text, " ".repeat(new_indent_size));
                    let line_segment_after_label = line_segment.trim().get(label_size..).unwrap();
                    new_line_segment = format!("{}{}", new_indent, line_segment_after_label.trim());

                    visual_region_length = label_size
                        + line_segment_after_label
                            .chars()
                            .take_while(|c| *c == ' ')
                            .count();
                }

                // Only populate edit string if the line_segment has changed
                if line_segment != new_line_segment.clone() {
                    if is_first_segment {
                        edit_string = new_line_segment;
                    } else {
                        edit_string = format!("{}\n{}", edit_string, new_line_segment);
                    }
                }
                // Remove semicolons that are not inside quotes
                let mut enclosing_quote: Option<char> = None;
                edit_string = edit_string
                    .chars()
                    .filter(|ch| {
                        enclosing_quote = update_enclosing_quote(ch, enclosing_quote);
                        !matches!(ch, ';') || enclosing_quote.is_some()
                    })
                    .collect();
            }

            is_first_segment = false;
        }

        if !edit_string.is_empty() {
            let expected_indent = edit_string.chars().take_while(|c| *c == ' ').count();

            let visual_end = if !line_contains_semicolon {
                line_segment_start
                    + TextSize::try_from(std::cmp::max(visual_region_length, 1)).unwrap()
            } else {
                line.end()
            };

            let range = TextRange::new(line.start(), visual_end);
            let fix = Fix::safe_edit(Edit::range_replacement(edit_string, line.range()));

            if is_preproc_violation {
                if let Some(diagnostic) =
                    context.create_diagnostic_if_enabled(IndentedPreprocessorStatement, range)
                {
                    violations.push(diagnostic.with_fix(fix));
                };
            } else if let Some(diagnostic) = context.create_diagnostic_if_enabled(
                IncorrectIndentation {
                    expected_indent,
                    semicolon_found: line_contains_semicolon,
                    label_found: line_has_label,
                },
                range,
            ) {
                violations.push(diagnostic.with_fix(fix));
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
    pub struct IncorrectIndentationSettings {
        pub ignore_semicolons: bool,
        pub program_indent: usize,
        pub module_indent: usize,
        pub procedure_indent: usize,
        pub derived_type_indent: usize,
        pub control_flow_indent: usize,
        pub interface_indent: usize,
        pub line_continuation_indent: usize,
    }

    impl Default for IncorrectIndentationSettings {
        fn default() -> Self {
            Self {
                ignore_semicolons: true,
                program_indent: 4usize,
                module_indent: 4usize,
                procedure_indent: 4usize,
                derived_type_indent: 4usize,
                control_flow_indent: 4usize,
                interface_indent: 4usize,
                line_continuation_indent: 4usize,
            }
        }
    }

    pub struct IndentMap(HashMap<&'static str, usize>);

    impl IndentMap {
        pub fn get_indent(&self, construct: &str) -> usize {
            *self.0.get(construct).unwrap()
        }
    }

    static PROGRAM_STATEMENTS: &[&str] = &["program_statement"];
    static MODULE_STATEMENTS: &[&str] = &["module_statement", "submodule_statement"];
    static PROCEDURE_STATEMENTS: &[&str] =
        &["subroutine_statement", "function_statement", "function"];
    static TYPE_STATEMENTS: &[&str] = &["derived_type_statement"];
    static CONTROL_FLOW_STATEMENTS: &[&str] = &[
        "block_construct",
        "if_statement",
        "where_statement",
        "select_case_statement",
        "select_type_statement",
        "select_rank_statement",
        "do_loop",
        "do_statement",
        "associate_statement",
    ];
    static INTERFACE_STATEMENTS: &[&str] = &["interface_statement"];
    static LINE_CONTINUATION_STATEMENTS: &[&str] = &["line_continuation"];

    impl IncorrectIndentationSettings {
        pub fn default_from_indent_width(indent_width: usize) -> Self {
            Self {
                ignore_semicolons: true,
                program_indent: indent_width,
                module_indent: indent_width,
                procedure_indent: indent_width,
                derived_type_indent: indent_width,
                control_flow_indent: indent_width,
                interface_indent: indent_width,
                line_continuation_indent: indent_width,
            }
        }

        pub fn indent_map(&self) -> IndentMap {
            let mut construct_to_indent_map = HashMap::new();

            for (statements, value) in [
                (PROGRAM_STATEMENTS, self.program_indent),
                (MODULE_STATEMENTS, self.module_indent),
                (PROCEDURE_STATEMENTS, self.procedure_indent),
                (TYPE_STATEMENTS, self.derived_type_indent),
                (CONTROL_FLOW_STATEMENTS, self.control_flow_indent),
                (INTERFACE_STATEMENTS, self.interface_indent),
                (LINE_CONTINUATION_STATEMENTS, self.line_continuation_indent),
            ] {
                for node in statements {
                    construct_to_indent_map.insert(*node, value);
                }
            }

            IndentMap(construct_to_indent_map)
        }
    }

    impl Display for IncorrectIndentationSettings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.incorrect-indentation",
                fields = [
                    self.ignore_semicolons,
                    self.program_indent,
                    self.module_indent,
                    self.procedure_indent,
                    self.derived_type_indent,
                    self.control_flow_indent,
                    self.interface_indent,
                    self.line_continuation_indent,
                ]
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::prelude::CommandCargoExt;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;
    use test_case::test_case;

    #[test_case(
        2,
        "!> My program\nprogram test\n  implicit none\nend program test"
        ; "indent 2"
    )]
    #[test_case(0, "!> My program\nprogram test\nimplicit none\nend program test"; "indent 0" )]
    fn program_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = "!> My program\nprogram test\nimplicit none\nend program test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            program-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        "!> My module\nmodule test\n  implicit none\ncontains\nend module test"
        ; "indent 2"
    )]
    #[test_case(
        0,
        "!> My module\nmodule test\nimplicit none\ncontains\nend module test"
        ; "indent 0"
    )]
    fn module_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = "!> My module\nmodule test\nimplicit none\ncontains\nend module test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            module-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        "!> My submodule\nsubmodule (mmod) test\n  implicit none\ncontains\nend submodule test"; "indent 2"
    )]
    #[test_case(
        0,
        "!> My submodule\nsubmodule (mmod) test\nimplicit none\ncontains\nend submodule test"; "indent 0"
    )]
    fn submodule_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet =
            "!> My submodule\nsubmodule (mmod) test\nimplicit none\ncontains\nend submodule test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            module-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        "!> My subroutine\nsubroutine test\n  implicit none\nend subroutine test"; "indent 2"
    )]
    #[test_case(
        0,
        "!> My subroutine\nsubroutine test\nimplicit none\nend subroutine test"; "indent 0"
    )]
    fn subroutine_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = "!> My subroutine\nsubroutine test\nimplicit none\nend subroutine test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            procedure-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test",
        "!> My function\nfunction test result(output)\n  integer :: output\nend function test"
        ; "indent 2 with result"
    )]
    #[test_case(
        0,
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test",
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test"
        ; "indent 0 with result"
    )]
    #[test_case(
        3,
        "!> My function\ninteger function test\ntest = 3\nend function test",
        "!> My function\ninteger function test\n   test = 3\nend function test"
        ; "indent 3 no result"
    )]
    #[test_case(
        0,
        "!> My function\ninteger function test\ntest = 3\nend function test",
        "!> My function\ninteger function test\ntest = 3\nend function test"
        ; "indent 0 no result"
    )]
    #[test_case(
        2,
        r#"
submodule (mmod) msubmodule
contains
module function interfaced_function(i) result(x)
integer, intent(in) :: i
x = i
end function interfaced_function
end submodule msubmodule"#,
        r#"
submodule (mmod) msubmodule
contains
    module function interfaced_function(i) result(x)
      integer, intent(in) :: i
      x = i
    end function interfaced_function
end submodule msubmodule"#
; "Interfaced function with result"
    )]
    #[test_case(
        3,
        r#"
submodule (mmod) msubmodule
contains
integer module function interfaced_function(i)
interfaced_function = i
end function interfaced_function
end submodule msubmodule"#,
        r#"
submodule (mmod) msubmodule
contains
    integer module function interfaced_function(i)
       interfaced_function = i
    end function interfaced_function
end submodule msubmodule"#
            ; "Interfaced function with return type"
    )]
    fn function_indentation(num_indents: i8, snippet: &str, fixed_snippet: &str) -> Result<()> {
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            procedure-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        r#"
module mmod
    type :: mtype
      integer :: i
    contains
      procedure :: mproc
    end type mtype
contains
end module mmod"#; "indent 2"
    )]
    #[test_case(
        0,
        r#"
module mmod
    type :: mtype
    integer :: i
    contains
    procedure :: mproc
    end type mtype
contains
end module mmod"#; "indent 0"
    )]
    fn derived_type_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
module mmod
type :: mtype
integer :: i
contains
procedure :: mproc
end type mtype
contains
end module mmod"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            derived-type-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        8,
        r#"
program mprog
    block
            real :: x = 3.142
            print*, x
            y = x
            inner: block
                    real :: y = 12.1
                    print*, y
            end block inner
    end block
end program mprog"#; "indent 8"
    )]
    #[test_case(
        0,
        r#"
program mprog
    block
    real :: x = 3.142
    print*, x
    y = x
    inner: block
    real :: y = 12.1
    print*, y
    end block inner
    end block
end program mprog"#; "indent 0"
    )]
    fn block_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
program mprog
block
real :: x = 3.142
print*, x
y = x
inner: block
real :: y = 12.1
print*, y
end block inner
end block
end program mprog"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            control-flow-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        false,
        4,
        r#"
subroutine msub()
    integer :: i
    i = i + 1
    ! inline if
    if (i == 1) i = 2
    ! Semicolons
    if (i == 2) then
        i = 3
    end if
    if (i == 4) then
        i = 2
    else if (i == 2) then
        i = 4
    else
        i = 1
    end if
    ! Named if block
    named_if: if (i == 1) then
        i = i + 1
    end if
end subroutine msub"#
    ; "All types of if with one indent and including semicolons"
    )]
    #[test_case(
        true,
        3,
        r#"
subroutine msub()
    integer :: i
    i = i + 1
    ! inline if
    if (i == 1) i = 2
    ! Semicolons
if (i == 2) then; i = 3; end if;
    if (i == 4) then
       i = 2
    else if (i == 2) then
       i = 4
    else
       i = 1
    end if
    ! Named if block
    named_if: if (i == 1) then
       i = i + 1
    end if
end subroutine msub"#
        ; "All types of if with three indent and ignoring semicolons"
    )]
    #[test_case(
        true,
        0,
        r#"
subroutine msub()
    integer :: i
    i = i + 1
    ! inline if
    if (i == 1) i = 2
    ! Semicolons
if (i == 2) then; i = 3; end if;
    if (i == 4) then
    i = 2
    else if (i == 2) then
    i = 4
    else
    i = 1
    end if
    ! Named if block
    named_if: if (i == 1) then
    i = i + 1
    end if
end subroutine msub"#
            ; "All types of if with zero indent and ignoring semicolons"
    )]
    fn if_indentation(ignore_semicolons: bool, num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
subroutine msub()
integer :: i
i = i + 1
! inline if
if (i == 1) i = 2
! Semicolons
if (i == 2) then; i = 3; end if;
if (i == 4) then
i = 2
else if (i == 2) then
i = 4
else
i = 1
end if
! Named if block
named_if: if (i == 1) then
i = i + 1
end if
end subroutine msub"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            ignore-semicolons = {}
            control-flow-indent = {}
            "#,
            ignore_semicolons, num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        r#"
module mmod
    interface
      module function interfaced_function(i) result(x)
          integer, intent(in) :: i
      end function interfaced_function
    end interface
    interface minterface
      module procedure minterface_i,minterface_r
    end interface minterface
end module mmod"#; "indent 2"
    )]
    #[test_case(
        0,
        r#"
module mmod
    interface
    module function interfaced_function(i) result(x)
        integer, intent(in) :: i
    end function interfaced_function
    end interface
    interface minterface
    module procedure minterface_i,minterface_r
    end interface minterface
end module mmod"#; "indent 0"
    )]
    fn interface_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
module mmod
interface
module function interfaced_function(i) result(x)
integer, intent(in) :: i
end function interfaced_function
end interface
interface minterface
module procedure minterface_i,minterface_r
end interface minterface
end module mmod"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            interface-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        3,
        r#"
subroutine select_cases
    integer :: i
    select case (i)
    case (1)
       i = 2
    case (2)
       i = 1
    end select
    i = 3
end subroutine select_cases"#; "indent 3"
    )]
    #[test_case(
        0,
        r#"
subroutine select_cases
    integer :: i
    select case (i)
    case (1)
    i = 2
    case (2)
    i = 1
    end select
    i = 3
end subroutine select_cases"#; "indent 0"
    )]
    fn select_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
subroutine select_cases
integer :: i
select case (i)
case (1)
i = 2
case (2)
i = 1
end select
i = 3
end subroutine select_cases"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            control-flow-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        r#"
function do_construct
    integer :: i, j, x
    do i = 1, 10
      do j = i, 10
        x = i * j
      end do
    end do
    named_do: do i = 1, 10
      print *, i
    end do
end function do_construct"#; "indent 2"
    )]
    #[test_case(
        0,
        r#"
function do_construct
    integer :: i, j, x
    do i = 1, 10
    do j = i, 10
    x = i * j
    end do
    end do
    named_do: do i = 1, 10
    print *, i
    end do
end function do_construct"#; "indent 0"
    )]
    fn do_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
function do_construct
integer :: i, j, x
do i = 1, 10
do j = i, 10
x = i * j
end do
end do
named_do: do i = 1, 10
print *, i
end do
end function do_construct"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            control-flow-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        2,
        r#"
subroutine associates
    integer :: i
    associate(x => i)
      print *, x
    end associate
    named_associate: associate(x => i)
      print *, x
    end associate named_associate
end subroutine associates"#; "indent 2"
    )]
    #[test_case(
        0,
        r#"
subroutine associates
    integer :: i
    associate(x => i)
    print *, x
    end associate
    named_associate: associate(x => i)
    print *, x
    end associate named_associate
end subroutine associates"#; "indent 0"
    )]
    fn associate_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
subroutine associates
integer :: i
associate(x => i)
print *, x
end associate
named_associate: associate(x => i)
print *, x
end associate named_associate
end subroutine associates"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            control-flow-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        8,
        r#"
function wrapped_function( &
        i &
)
    integer, intent(in) :: i
    print *, x
    i = i + 1 &
            + 2 &
            + 3
end function wrapped_function"#; "indent 2"
    )]
    #[test_case(
        0,
        r#"
function wrapped_function( &
i &
)
    integer, intent(in) :: i
    print *, x
    i = i + 1 &
    + 2 &
    + 3
end function wrapped_function"#; "indent 0"
    )]
    fn line_continuation_indentation(num_indents: i8, fixed_snippet: &str) -> Result<()> {
        let snippet = r#"
function wrapped_function( &
i &
)
integer, intent(in) :: i
print *, x
i = i + 1 &
+ 2 &
+ 3
end function wrapped_function"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            line-continuation-indent = {}
            "#,
            num_indents,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test]
    fn where_statement() -> Result<()> {
        let before = r#"
subroutine wheresub
    real :: pressure(1000), temp(1000), precepitation(1000)

    where(pressure >= 1.0)
    pressure = pressure + 1.0
    temp = temp - 10.0
    elsewhere
    precepitation = .TRUE.
    endwhere
end subroutine wheresub
"#;

        let expected = r#"
subroutine wheresub
    real :: pressure(1000), temp(1000), precepitation(1000)

    where(pressure >= 1.0)
        pressure = pressure + 1.0
        temp = temp - 10.0
    elsewhere
        precepitation = .TRUE.
    endwhere
end subroutine wheresub
"#;
        verify_s105_fixes(before, expected, "")
    }

    #[test]
    fn select_type_statement() -> Result<()> {
        let before = r#"
subroutine print_decorated_numbers(number)
  class(*), intent(in) :: number

  select type(number)
   type is (integer)
      print*, 'integer'
      type is (real)
          print*, 'real'
          class is (custom_type)
              print*, 'custom'
            class default
              print*, 'not a number'
            end select
end subroutine print_decorated_numbers
"#;

        let expected = r#"
subroutine print_decorated_numbers(number)
    class(*), intent(in) :: number

    select type(number)
    type is (integer)
        print*, 'integer'
    type is (real)
        print*, 'real'
    class is (custom_type)
        print*, 'custom'
    class default
        print*, 'not a number'
    end select
end subroutine print_decorated_numbers
"#;
        verify_s105_fixes(before, expected, "")
    }

    #[test]
    fn select_rank_statement() -> Result<()> {
        let before = r#"
subroutine assumed_rank(A)
  integer, intent(inout) :: A(..)

select rank(A)
   rank (0)
      write(*, *) "scalar"
     rank (1)
        write(*, *) "rank 1"
       rank default
          error stop 'assumed_rank: only rank 0..2 is handled for now.'
         end select
end subroutine
"#;

        let expected = r#"
subroutine assumed_rank(A)
    integer, intent(inout) :: A(..)

    select rank(A)
    rank (0)
        write(*, *) "scalar"
    rank (1)
        write(*, *) "rank 1"
    rank default
        error stop 'assumed_rank: only rank 0..2 is handled for now.'
    end select
end subroutine
"#;
        verify_s105_fixes(before, expected, "")
    }

    fn verify_s105_fixes(snippet: &str, fixed_snippet: &str, toml_contents: &str) -> Result<()> {
        let temp_dir = TempDir::new()?;
        let fixed_src_path = temp_dir.path().join("S105-fixed.f90");
        let toml_path = temp_dir.path().join("fortitude.toml");
        fs::write(&fixed_src_path, snippet)?;
        fs::write(&toml_path, toml_contents)?;

        Command::cargo_bin("fortitude")?
            .arg("check")
            .arg("--config")
            .arg(toml_path.as_os_str())
            .arg("--fix")
            .arg("--preview")
            .arg("--select=S105")
            .arg(fixed_src_path.as_os_str())
            .status()?;
        let fixed: String = String::from_utf8(fs::read(fixed_src_path.as_os_str())?)?;
        assert_eq!(fixed_snippet, fixed);

        Ok(())
    }
}
