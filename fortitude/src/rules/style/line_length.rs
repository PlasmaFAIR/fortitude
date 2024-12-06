/// Defines rules that govern line length.
use crate::settings::Settings;
use crate::TextRule;
use lazy_regex::regex_is_match;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::OneIndexed;
use ruff_source_file::SourceFile;
use ruff_text_size::{TextLen, TextRange, TextSize};

/// ## What does it do?
/// Checks line length isn't too long
///
/// ## Why is this bad?
/// Long lines are more difficult to read, and may not fit on some developers'
/// terminals. The line continuation character '&' may be used to split a long line
/// across multiple lines, and overly long expressions may be broken down into
/// multiple parts.
///
/// The maximum line length can be changed using the flag `--line-length=N`. The
/// default maximum line length is 100 characters. This is a fair bit more than the
/// traditional 80, but due to the verbosity of modern Fortran it can sometimes be
/// difficult to squeeze lines into that width, especially when using large indents
/// and multiple levels of indentation.
///
/// Note that the Fortran standard states a maximum line length of 132 characters,
/// and while some modern compilers will support longer lines, for portability it
/// is recommended to stay beneath this limit.
#[violation]
pub struct LineTooLong {
    max_length: usize,
    actual_length: usize,
}

impl Violation for LineTooLong {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            max_length,
            actual_length,
        } = self;
        format!("line length of {actual_length}, exceeds maximum {max_length}")
    }
}

impl TextRule for LineTooLong {
    fn check(settings: &Settings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let max_length = settings.line_length;
        let mut violations = Vec::new();
        for (idx, line) in source.text().lines().enumerate() {
            // Note: Can't use string.len(), as that gives byte length, not char length
            let actual_length = line.chars().count();
            if actual_length > max_length {
                // Are we ending on a string or comment? If so, we'll allow it through, as it may
                // contain something like a long URL that cannot be reasonably split across multiple
                // lines.
                if regex_is_match!(r#"(["']\w*&?$)|(!.*$)|(^\w*&)"#, line) {
                    continue;
                }
                // Get the byte range from the first character that oversteps the limit
                // to the end of the line
                let extra_bytes: TextSize = line
                    .chars()
                    .rev()
                    .take(actual_length - max_length)
                    .map(TextLen::text_len)
                    .sum();
                let line_end = source.line_end_exclusive(OneIndexed::from_zero_indexed(idx));
                let range = TextRange::new(line_end - extra_bytes, line_end);
                violations.push(Diagnostic::new(
                    Self {
                        max_length,
                        actual_length,
                    },
                    range,
                ));
            }
        }
        violations
    }
}
