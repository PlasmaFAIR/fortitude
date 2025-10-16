use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

const FORTRAN_VALID_CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_ =+-*/\\()[]{},.:;!\"%&~<>?\'`^|$#@\n\r\t";

/// ## What it does
/// Checks for the use of invalid characters in source code (except strings and
/// comments)
///
/// ## Why is this bad?
/// The Fortran standard only supports the basic ASCII character set (`a-z, A-Z,
/// 0-9`, and some punctuation), and all the main compilers will error on
/// non-ASCII characters, for example letters with diacritics or accents (except
/// in comments or string literals).
#[derive(ViolationMetadata)]
pub(crate) struct InvalidCharacter {
    character: char,
}

impl Violation for InvalidCharacter {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { character } = self;
        format!("Invalid character '{character}'")
    }
}

pub fn check_invalid_character(root: &Node, src: &SourceFile) -> Vec<Diagnostic> {
    src.source_text()
        .char_indices()
        .filter(|(_, c)| !FORTRAN_VALID_CHARACTERS.contains(*c))
        .filter(|(index, _)| {
            if let Some(node) = root.named_descendant_for_byte_range(*index, *index) {
                !matches!(node.kind(), "comment" | "string_literal")
            } else {
                false
            }
        })
        .map(|(index, character)| {
            let start = TextSize::try_from(index).unwrap();
            let range = TextRange::new(start, start);
            Diagnostic::new(InvalidCharacter { character }, range)
        })
        .collect_vec()
}
