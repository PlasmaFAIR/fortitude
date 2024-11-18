use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;

use crate::settings::Settings;
use crate::{FromStartEndLineCol, TextRule};
/// Defines rules that enforce widely accepted whitespace rules.

/// ## What does it do?
/// Checks for tailing whitespace
///
/// ## Why is this bad?
/// Trailing whitespace is difficult to spot, and as some editors will remove it
/// automatically while others leave it, it can cause unwanted 'diff noise' in
/// shared projects.
#[violation]
pub struct TrailingWhitespace {}

impl Violation for TrailingWhitespace {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("trailing whitespace")
    }
}
impl TextRule for TrailingWhitespace {
    fn check(_settings: &Settings, source: &SourceFile) -> Vec<Diagnostic> {
        let mut violations = Vec::new();
        for (idx, line) in source.source_text().split('\n').enumerate() {
            if line.ends_with([' ', '\t']) {
                violations.push(Diagnostic::from_start_end_line_col(
                    Self {},
                    source,
                    idx,
                    line.trim_end().len(),
                    idx,
                    line.len(),
                ));
            }
        }
        violations
    }
}
