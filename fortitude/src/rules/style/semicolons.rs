use lazy_regex::bytes_regex_is_match;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::{TextRange, TextSize};

use crate::rules::text::blank_comments_and_strings;
use crate::settings::Settings;
use crate::TextRule;

fn semicolon_is_superfluous<S: AsRef<str>>(line: S, position: usize) -> bool {
    let line = line.as_ref();
    // A semicolons is superfluous if:
    // - It is at the beginning of a line, possibly containing a line continuation or
    //   other semicolons.
    // - It is at the end of the last statement on a line, even if followed by a line
    //   continuation character.
    // - It is followed by other semicolons (with any amount of whitespace in between)
    line.as_bytes()[..position]
        .iter()
        .all(|b| b.is_ascii_whitespace() || *b == b'&' || *b == b';')
        || bytes_regex_is_match!(r"^;[\s!&]*$", &line.as_bytes()[position..])
        || bytes_regex_is_match!(r"^;\s*;", &line.as_bytes()[position..])
}

/// ## What does it do?
/// Catches a semicolon at the end of a line of code.
///
/// ## Why is this bad?
/// Many languages use semicolons to denote the end of a statement, but in Fortran each
/// line of code is considered its own statement (unless it ends with a line
/// continuation character, `'&'`). Semicolons may be used to separate multiple
/// statements written on the same line, but a semicolon at the end of a line has no
/// effect.
///
/// A semicolon at the beginning of a statement similarly has no effect, nor do
/// multiple semicolons in sequence.
#[violation]
pub struct SuperfluousSemicolon {}

impl AlwaysFixableViolation for SuperfluousSemicolon {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("unnecessary semicolon")
    }

    fn fix_title(&self) -> String {
        format!("Remove this character")
    }
}

impl TextRule for SuperfluousSemicolon {
    fn check(_settings: &Settings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let text = blank_comments_and_strings(source.text());
        text.lines()
            .enumerate()
            .flat_map(|(line_idx, line)| {
                let line_start_byte = source.line_start(OneIndexed::from_zero_indexed(line_idx));
                line.bytes()
                    .enumerate()
                    .filter_map(move |(col_idx, b)| {
                        if b == b';' && semicolon_is_superfluous(line, col_idx) {
                            Some(col_idx)
                        } else {
                            None
                        }
                    })
                    .map(move |col_idx| {
                        let leading_whitespace = line.as_bytes()[..col_idx]
                            .iter()
                            .rev()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        let trailing_whitespace = line.as_bytes()[col_idx + 1..]
                            .iter()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        let edit_start =
                            line_start_byte + TextSize::from((col_idx - leading_whitespace) as u32);
                        let edit_end = line_start_byte
                            + TextSize::from((col_idx + 1 + trailing_whitespace) as u32);
                        let edit = Edit::deletion(edit_start, edit_end);
                        let report_start = line_start_byte + TextSize::from(col_idx as u32);
                        let report_end = line_start_byte + TextSize::from((col_idx + 1) as u32);
                        let range = TextRange::new(report_start, report_end);
                        Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit))
                    })
            })
            .collect()
    }
}

/// ## What does it do?
/// Catches multiple statements on the same line separated by a semicolon.
///
/// ## Why is this bad?
/// This can have a detrimental effect on code readability.
#[violation]
pub struct MultipleStatementsPerLine {}

impl AlwaysFixableViolation for MultipleStatementsPerLine {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("multiple statements per line")
    }

    fn fix_title(&self) -> String {
        format!("Separate over two lines")
    }
}

impl TextRule for MultipleStatementsPerLine {
    fn check(_settings: &Settings, source_file: &SourceFile) -> Vec<Diagnostic> {
        let source = source_file.to_source_code();
        let text = blank_comments_and_strings(source.text());
        text.lines()
            .enumerate()
            .flat_map(|(line_idx, line)| {
                let line_start_byte = source.line_start(OneIndexed::from_zero_indexed(line_idx));
                line.bytes()
                    .enumerate()
                    .filter_map(move |(col_idx, b)| {
                        if b == b';' && !semicolon_is_superfluous(line, col_idx) {
                            Some(col_idx)
                        } else {
                            None
                        }
                    })
                    .map(move |col_idx| {
                        let leading_whitespace = line.as_bytes()[..col_idx]
                            .iter()
                            .rev()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        let trailing_whitespace = line.as_bytes()[col_idx + 1..]
                            .iter()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        let indentation: String =
                            line.chars().take_while(|c| c.is_whitespace()).collect();
                        let replacement = format!("\n{indentation}");
                        let edit_start =
                            line_start_byte + TextSize::from((col_idx - leading_whitespace) as u32);
                        let edit_end = line_start_byte
                            + TextSize::from((col_idx + 1 + trailing_whitespace) as u32);
                        let edit = Edit::replacement(replacement, edit_start, edit_end);
                        let report_start = line_start_byte + TextSize::from(col_idx as u32);
                        let report_end = line_start_byte + TextSize::from((col_idx + 1) as u32);
                        let range = TextRange::new(report_start, report_end);
                        Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit))
                    })
            })
            .collect()
    }
}
