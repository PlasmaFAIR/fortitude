/// Defines rules that govern line length.
use crate::settings::CheckSettings;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_source_file::UniversalNewlines;
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
/// In the interest of pragmatism, this rule makes a few exceptions when
/// determining whether a line is overlong. Namely, it:
///
/// 1. Ignores lines that consist of a single "word" (that is, without any
///    whitespace between its characters).
/// 2. Ignores lines that end with a URL, as long as the URL starts before
///    the line-length threshold.
/// 3. Ignores SPDX license identifiers and copyright notices (for example, `!
///    SPDX-License-Identifier: MIT`), which are machine-readable and should
///    _not_ wrap over multiple lines.
///
/// Note that the Fortran standard states a maximum line length of 132
/// characters[^1], and while most modern compilers will support longer lines[^2],
/// for portability it is recommended to stay beneath this limit.
///
/// ## Example
/// ```f90
/// call my_long_subroutine(param1, param2, param3, param4, param5, param6, param7, param8, param9, param10)
/// ```
///
/// Use instead:
/// ```f90
/// call my_long_subroutine(&
///     param1, param2, param3, param4, param5, &
///     param6, param7, param8, param9, param10 &
/// )
/// ```
///
/// ## Options
/// - `check.line-length`
///
/// [^1]: In F77 this was only 72, and in F2023 it was relaxed to 10,000.
/// [^2]: Sometimes a compiler flag is required.
#[derive(ViolationMetadata)]
pub(crate) struct LineTooLong {
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

impl LineTooLong {
    pub fn check(settings: &CheckSettings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let limit = settings.line_length;
        let mut violations = Vec::new();

        let tab_size = settings.invalid_tab.indent_width.as_usize();

        for line in source.text().universal_newlines() {
            // The maximum width of the line is the number of bytes multiplied by the tab size (the
            // worst-case scenario is that the line is all tabs). If the maximum width is less than the
            // limit, then the line is not overlong.
            let max_possible = line.len() * tab_size;
            if max_possible < limit {
                continue;
            }

            // Note: Can't use string.len(), as that gives byte length, not char length
            let width = measure(&line, tab_size);
            if width < limit {
                continue;
            }

            let mut chunks = line.split_whitespace();
            let (Some(first_chunk), Some(second_chunk)) = (chunks.next(), chunks.next()) else {
                // Single word / no printable chars - no way to make the line shorter.
                continue;
            };

            // Do not enforce the line length for lines that end with a URL, as long as the URL
            // begins before the limit.
            let last_chunk = chunks.last().unwrap_or(second_chunk);
            if last_chunk.contains("://") && width - measure(last_chunk, tab_size) <= limit {
                continue;
            }

            // Do not enforce the line length limit for SPDX license headers, which are machine-readable
            // and explicitly _not_ recommended to wrap over multiple lines.
            if matches!(
                (first_chunk, second_chunk),
                ("!", "SPDX-License-Identifier:" | "SPDX-FileCopyrightText:")
            ) {
                continue;
            }

            // Get the byte range from the first character that oversteps the limit
            // to the end of the line
            let extra_bytes: TextSize = line
                .chars()
                .rev()
                .take(width - limit)
                .map(TextLen::text_len)
                .sum();
            let range = TextRange::new(line.end() - extra_bytes, line.end());
            violations.push(Diagnostic::new(
                Self {
                    max_length: limit,
                    actual_length: width,
                },
                range,
            ));
        }
        violations
    }
}

/// Returns the width of a given string, accounting for the tab size.
// TODO: actually take into account tab width
fn measure(s: &str, _tab_size: usize) -> usize {
    s.chars().count()
}
