/// Defines rules that enforce widely accepted whitespace rules.
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use crate::logical_lines::LogicalLines;
use crate::rules::Rule;
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;
use ruff_source_file::{OneIndexed, UniversalNewlines};
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
/// Checks that the correct indentation has been used.
///
/// ## Why is this bad?
/// Inconsistent indentation makes Fortran less readable and difficult to
/// understand the scoping of logic.
///
/// The expected indentation is inferred on a per-file basis, or it can be set
/// using the `check.indent-width` option. Continued lines are ignored, as
/// they are often indented differently for readability.
///
/// ## Options
/// - `check.indent-width`
/// - `check.incorrect-indentation.indent-programs`
/// - `check.incorrect-indentation.indent-modules`
/// - `check.incorrect-indentation.indent-procedures`
/// - `check.incorrect-indentation.indent-derived-types`
/// - `check.incorrect-indentation.indent-control-flow`
/// - `check.incorrect-indentation.indent-interfaces`
#[derive(ViolationMetadata)]
pub(crate) struct IncorrectIndentation {
    actual: usize,
    expected: usize,
}

impl AlwaysFixableViolation for IncorrectIndentation {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { expected, actual } = self;
        format!("Incorrect indentation; expected {expected} spaces, found {actual}")
    }

    fn fix_title(&self) -> String {
        "Replace with correct number of spaces".to_string()
    }
}

/// ## What it does
/// Checks that preprocessor statements have zero indentation before the '#'.
///
/// ## Why is this bad?
/// Preprocessor statements with indentation are invalid for most Fortran
/// compilers and can lead to compilation errors.
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

pub(crate) fn check_incorrect_indent(
    context: &CheckContext,
    lines: &LogicalLines,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();
    let source_code = context.source_file().to_source_code();

    // Store settings
    // TODO: Get the indent width from the context instead of the settings if not provided
    let indent_width = context.settings().indent_width;

    for line in lines.iter() {
        let indentation = line.indentation();
        let start_byte = line.start_byte();

        // Tabs are not valid Fortran, and should be handled elsewhere
        if indentation.contains('\t') {
            continue;
        }

        // Lines after semicolons should be ignored.
        if source_code.line_column(start_byte).column > OneIndexed::from_zero_indexed(0) {
            continue;
        }

        let expected_indent = line.expected_indent(indent_width.as_u8());
        let actual_indent = indentation.len();

        if actual_indent != expected_indent as usize {
            let indentation_range = TextRange::new(
                start_byte,
                start_byte + TextSize::from(actual_indent as u32),
            );
            let preproc = line.text().trim_start().starts_with('#');
            if preproc && context.is_rule_enabled(Rule::IndentedPreprocessorStatement) {
                let edit = Edit::range_deletion(indentation_range);
                violations.push(
                    context
                        .create_diagnostic(IndentedPreprocessorStatement {}, indentation_range)
                        .with_fix(Fix::safe_edit(edit)),
                );
            }
            if !preproc && context.is_rule_enabled(Rule::IncorrectIndentation) {
                let edit = if expected_indent > 0 {
                    Edit::range_replacement(" ".repeat(expected_indent as usize), indentation_range)
                } else {
                    Edit::range_deletion(indentation_range)
                };
                violations.push(
                    context
                        .create_diagnostic(
                            IncorrectIndentation {
                                actual: actual_indent,
                                expected: expected_indent as usize,
                            },
                            indentation_range,
                        )
                        .with_fix(Fix::safe_edit(edit)),
                );
            }
        }
    }
    violations
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::Display;

    #[derive(Debug, Clone, CacheKey)]
    pub struct IncorrectIndentationSettings {
        pub indent_programs: bool,
        pub indent_modules: bool,
        pub indent_procedures: bool,
        pub indent_derived_types: bool,
        pub indent_control_flow: bool,
        pub indent_interfaces: bool,
    }

    impl Default for IncorrectIndentationSettings {
        fn default() -> Self {
            Self {
                indent_programs: true,
                indent_modules: true,
                indent_procedures: true,
                indent_derived_types: true,
                indent_control_flow: true,
                indent_interfaces: true,
            }
        }
    }

    impl Display for IncorrectIndentationSettings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.incorrect-indentation",
                fields = [
                    self.indent_programs,
                    self.indent_modules,
                    self.indent_procedures,
                    self.indent_derived_types,
                    self.indent_control_flow,
                    self.indent_interfaces,
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
        true,
        "!> My program\nprogram test\n    implicit none\nend program test"
        ; "indent 1"
    )]
    #[test_case(false, "!> My program\nprogram test\nimplicit none\nend program test"; "indent 0" )]
    fn test_s105_program_indentation(indent_programs: bool, fixed_snippet: &str) -> Result<()> {
        let snippet = "!> My program\nprogram test\nimplicit none\nend program test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-programs = {}
            "#,
            indent_programs,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
        "!> My module\nmodule test\n    implicit none\ncontains\nend module test"
        ; "indent 1"
    )]
    #[test_case(
        false,
        "!> My module\nmodule test\nimplicit none\ncontains\nend module test"
        ; "indent 0"
    )]
    fn test_s105_module_indentation(indent_modules: bool, fixed_snippet: &str) -> Result<()> {
        let snippet = "!> My module\nmodule test\nimplicit none\ncontains\nend module test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-modules = {}
            "#,
            indent_modules,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
        "!> My submodule\nsubmodule (mmod) test\n    implicit none\ncontains\nend submodule test"; "indent 1"
    )]
    #[test_case(
        false,
        "!> My submodule\nsubmodule (mmod) test\nimplicit none\ncontains\nend submodule test"; "indent 0"
    )]
    fn test_s105_submodule_indentation(indent_modules: bool, fixed_snippet: &str) -> Result<()> {
        let snippet =
            "!> My submodule\nsubmodule (mmod) test\nimplicit none\ncontains\nend submodule test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-modules = {}
            "#,
            indent_modules,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
        "!> My subroutine\nsubroutine test\n    implicit none\nend subroutine test"; "indent 1"
    )]
    #[test_case(
        false,
        "!> My subroutine\nsubroutine test\nimplicit none\nend subroutine test"; "indent 0"
    )]
    fn test_s105_subroutine_indentation(
        indent_procedures: bool,
        fixed_snippet: &str,
    ) -> Result<()> {
        let snippet = "!> My subroutine\nsubroutine test\nimplicit none\nend subroutine test";
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-procedures = {}
            "#,
            indent_procedures,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test",
        "!> My function\nfunction test result(output)\n    integer :: output\nend function test"
        ; "indent 1 with result"
    )]
    #[test_case(
        false,
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test",
        "!> My function\nfunction test result(output)\ninteger :: output\nend function test"
        ; "indent 0 with result"
    )]
    #[test_case(
        true,
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
        true,
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
    fn test_s105_function_indentation(
        indent_procedures: bool,
        snippet: &str,
        fixed_snippet: &str,
    ) -> Result<()> {
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-procedures = {}
            "#,
            indent_procedures,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
        r#"
module mmod
    type :: mtype
        integer :: i
    contains
        procedure :: mproc
    end type mtype
contains
end module mmod"#; "indent 1"
    )]
    #[test_case(
        false,
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
    fn test_s105_derived_type_indentation(
        indent_derived_types: bool,
        fixed_snippet: &str,
    ) -> Result<()> {
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
            indent-derived-types = {}
            "#,
            indent_derived_types,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
end program mprog"#; "indent 2"
    )]
    #[test_case(
        false,
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
    fn test_s105_block_indentation(indent_control_flow: bool, fixed_snippet: &str) -> Result<()> {
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
            indent-control-flow = {}
            "#,
            indent_control_flow,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
        ; "indent 1"
    )]
    #[test_case(
        false,
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
            ; "indent 0"
    )]
    fn test_s105_if_indentation(indent_control_flow: bool, fixed_snippet: &str) -> Result<()> {
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
            indent-control-flow = {}
            "#,
            indent_control_flow,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
end module mmod"#; "indent 1"
    )]
    #[test_case(
        false,
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
    fn test_s105_interface_indentation(indent_interfaces: bool, fixed_snippet: &str) -> Result<()> {
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
            indent-interfaces = {}
            "#,
            indent_interfaces,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
end subroutine select_cases"#; "indent 1"
    )]
    #[test_case(
        false,
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
    fn test_s105_select_indentation(indent_control_flow: bool, fixed_snippet: &str) -> Result<()> {
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
            indent-control-flow = {}
            "#,
            indent_control_flow,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
    do 10 i = 1, 10
        do 10 j = i, 10
            x = i * j
    10 continue
end function do_construct"#; "indent 1"
    )]
    #[test_case(
        false,
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
    do 10 i = 1, 10
    do 10 j = i, 10
    x = i * j
    10 continue
end function do_construct"#; "indent 0"
    )]
    fn test_s105_do_indentation(indent_control_flow: bool, fixed_snippet: &str) -> Result<()> {
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
do 10 i = 1, 10
do 10 j = i, 10
x = i * j
10 continue
end function do_construct"#;
        let toml_contents = format!(
            r#"
            [check.incorrect-indentation]
            indent-control-flow = {}
            "#,
            indent_control_flow,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test_case(
        true,
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
        false,
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
    fn test_s105_associate_indentation(
        indent_control_flow: bool,
        fixed_snippet: &str,
    ) -> Result<()> {
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
            indent-control-flow = {}
            "#,
            indent_control_flow,
        );

        verify_s105_fixes(snippet, fixed_snippet, &toml_contents)
    }

    #[test]
    fn test_s105_where_statement() -> Result<()> {
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
    fn test_s105_select_type_statement() -> Result<()> {
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
    fn test_s105_select_rank_statement() -> Result<()> {
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
            .arg("--config-file")
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
