use crate::settings::Settings;
use crate::{FromStartEndLineCol, TextRule};
use lazy_regex::regex_is_match;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
/// Defines rules that govern line length.

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
    fn check(settings: &Settings, source: &SourceFile) -> Vec<Diagnostic> {
        let max_length = settings.line_length;
        let mut violations = Vec::new();
        for (idx, line) in source.source_text().split('\n').enumerate() {
            let actual_length = line.len();
            if actual_length > max_length {
                // Are we ending on a string or comment? If so, we'll allow it through, as it may
                // contain something like a long URL that cannot be reasonably split across multiple
                // lines.
                if regex_is_match!(r#"(["']\w*&?$)|(!.*$)|(^\w*&)"#, line) {
                    continue;
                }
                violations.push(Diagnostic::from_start_end_line_col(
                    Self {
                        max_length,
                        actual_length,
                    },
                    source,
                    idx,
                    max_length,
                    idx,
                    actual_length,
                ))
            }
        }
        violations
    }
}
