/// Defines rules that govern line length.
use crate::settings::Settings;
use crate::TextRule;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::OneIndexed;
use ruff_source_file::SourceFile;
use ruff_text_size::{TextLen, TextRange, TextSize};

/// ## What does it do?
/// Checks if a backslash is the last character on a line
///
/// ## Why is this bad?
/// When compilers use the C preprocessor to pre-process Fortran files
/// the \ character is treated as a line continuation character by the C preprocessor,
/// potentially causing lines to be merged into one.
///
/// ## Example
/// When this Fortran program is passed through the C preprocessor,
/// ```f90
/// program t
///     implicit none
///     real :: A
///
///     ! Just a comment \
///     A = 2.0
///     print *, A
///  end
/// ```
/// it will end up with the variable assignment A placed onto the comment line,
/// ```f90
/// program t
///    implicit none
///    real :: A
///
///    ! Just a comment    A = 2.0
///
///    print *, A
/// end
/// ```
/// which causes the assignment to not be compiled.
///
#[violation]
pub struct TrailingBackslash {}

impl Violation for TrailingBackslash {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Trailing backslash")
    }
}

impl TextRule for TrailingBackslash {
    fn check(_settings: &Settings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let mut violations = Vec::new();

        for (idx, line) in source.text().lines().enumerate() {
            if line.trim_end().ends_with("\\") {
                let len = line.trim_end().chars().count();

                // Skip to position the warning underneath the \, which is 1 before the end of the line
                let offset: TextSize = line.chars().skip(len - 1).map(TextLen::text_len).sum();
                let line_end = source.line_end_exclusive(OneIndexed::from_zero_indexed(idx));
                let range = TextRange::new(line_end - offset, line_end - offset);
                violations.push(Diagnostic::new(Self {}, range));
            }
        }
        violations
    }
}
