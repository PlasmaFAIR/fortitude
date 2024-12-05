/// Defines rules that enforce widely accepted whitespace rules.
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::{TextLen, TextRange, TextSize};

use crate::settings::Settings;
use crate::{FromStartEndLineCol, TextRule};

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
                let edit = Edit::range_deletion(TextRange::new(
                    line_end_byte - whitespace_bytes,
                    line_end_byte,
                ));
                violations.push(
                    Diagnostic::from_start_end_line_col(
                        Self {},
                        source_file,
                        idx,
                        line.trim_end().len(),
                        idx,
                        line.len(),
                    )
                    .with_fix(Fix::safe_edit(edit)),
                );
            }
        }
        violations
    }
}
