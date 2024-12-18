use lazy_regex::bytes_regex_is_match;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::{TextRange, TextSize};

use crate::rules::text::blank_comments_and_strings;
use crate::settings::Settings;
use crate::TextRule;

/// ## What does it do?
/// Multiple statements separated by a semi-colon on the same line.
///
/// ## Why is this bad?
/// This can have a detrimental effect on the readability of the code.
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
                    .filter_map(|(col_idx, b)| {
                        // Find semi-colons that aren't at the end of a line
                        if b == b';'
                            && !bytes_regex_is_match!(r"^;[\s!]*$", &line.as_bytes()[col_idx..])
                        {
                            Some(col_idx)
                        } else {
                            None
                        }
                    })
                    .map(move |col_idx| {
                        let trailing_whitespace = line.as_bytes()[col_idx + 1..]
                            .iter()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        let replacement = format!(
                            "\n{}",
                            line.chars()
                                .take_while(|c| c.is_whitespace())
                                .collect::<String>()
                        );
                        let start_byte = line_start_byte + TextSize::from(col_idx as u32);
                        let end_byte = line_start_byte
                            + TextSize::from((col_idx + 1 + trailing_whitespace) as u32);
                        let range = TextRange::new(start_byte, end_byte);
                        let edit = Edit::replacement(replacement, start_byte, end_byte);
                        Diagnostic::new(Self {}, range).with_fix(Fix::safe_edit(edit))
                    })
            })
            .collect()
    }
}
