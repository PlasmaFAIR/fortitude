use crate::settings::Settings;
use crate::TextRule;
use lazy_regex::regex;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};

/// ## What it does
/// Checks for Fortran-escaped quotes in string literals that have been split over two lines.
///
/// ## Why is this bad?
/// In Fortran string literals, literal (escaped) double or single quotes are denoted
/// with two characters: `""` or `''`. A surprising Fortran feature is the ability to
/// split tokens over multiple lines, including these escaped quotes. The result is that
/// it's possible to mistake such a split escaped quote with implicit concatenation of
/// string literals, a feature in other languages but not in Fortran. Splitting escaped
/// quotes is practically never desired, and can be safely replaced with a simple line
/// continuation.
///
/// ## Example
/// ```f90
/// print*, "this looks like implicit "&
///      &" concatenation but isn't"
/// end
/// ```
///
/// Use instead:
/// ```f90
/// print*, "this looks like implicit&
///      & concatenation but isn't"
/// end
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SplitEscapedQuote;

impl AlwaysFixableViolation for SplitEscapedQuote {
    #[derive_message_formats]
    fn message(&self) -> String {
        "line continuation in split escaped quote looks like implicit concatenation".to_string()
    }

    fn fix_title(&self) -> String {
        "remove escaped quote".to_string()
    }
}

// tree-sitter-fortran doesn't actually capture the first part of this kind of string as
// part of the string literal (it should do!), so we have to do a regex over the whole
// text. We're looking for something that spans two lines, so it has to search the whole
// text at once too.
impl TextRule for SplitEscapedQuote {
    fn check(_settings: &Settings, src: &SourceFile) -> Vec<Diagnostic> {
        let text = src.source_text();
        let split_quote_re = regex!(r#"(?m)(['\"])(& *\r?\n *&?)(['\"])"#);

        split_quote_re
            .captures_iter(text)
            .filter_map(|capture| {
                // regex crate doesn't support backreferences, so we have to do it manually
                let (_, [first, continuation, second]) = capture.extract();
                if first != second {
                    return None;
                }

                let whole = capture.get(0)?;
                let std::ops::Range { start, end } = whole.range();
                let range = TextRange::new(
                    TextSize::try_from(start).unwrap(),
                    TextSize::try_from(end).unwrap(),
                );

                let edit = Edit::range_replacement(continuation.to_string(), range);
                Some(Diagnostic::new(SplitEscapedQuote, range).with_fix(Fix::safe_edit(edit)))
            })
            .collect()
    }
}
