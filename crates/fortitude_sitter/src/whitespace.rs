// Adapted from ruff
// Copyright 2024 Charles Marsh
// SPDX-License-Identifier: MIT

use ruff_source_file::LineRanges;
use ruff_text_size::{TextRange, TextSize};

/// Extract the leading indentation from a line.
pub fn indentation_at_offset(offset: TextSize, source: &str) -> Option<&str> {
    let line_start = source.line_start(offset);
    let indentation = &source[TextRange::new(line_start, offset)];

    indentation
        .chars()
        .all(is_fortran_whitespace)
        .then_some(indentation)
}

/// Return `true` if the node starting the given [`TextSize`] has leading content.
pub fn has_leading_content(offset: TextSize, source: &str) -> bool {
    let line_start = source.line_start(offset);
    let leading = &source[TextRange::new(line_start, offset)];
    leading.chars().any(|char| !is_fortran_whitespace(char))
}

/// Return `true` if the node ending at the given [`TextSize`] has trailing
/// content.
pub fn has_trailing_content(offset: TextSize, source: &str) -> bool {
    let line_end = source.line_end(offset);
    let trailing = &source[TextRange::new(offset, line_end)];
    trailing.chars().any(|char| !is_fortran_whitespace(char))
}

/// Returns `true` for Fortran whitespace characters.
pub const fn is_fortran_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

/// Extract the leading indentation from a line.
pub fn leading_indentation(line: &str) -> &str {
    line.find(|char: char| !is_fortran_whitespace(char))
        .map_or(line, |index| &line[..index])
}

pub trait FortranWhitespace {
    /// Like `str::trim()`, but only removes whitespace characters that Fortran considers
    /// to be whitespace.
    fn trim_whitespace(&self) -> &Self;

    /// Like `str::trim_start()`, but only removes whitespace characters that Fortran considers
    /// to be whitespace.
    fn trim_whitespace_start(&self) -> &Self;

    /// Like `str::trim_end()`, but only removes whitespace characters that Fortran considers
    /// to be whitespace.
    fn trim_whitespace_end(&self) -> &Self;
}

impl FortranWhitespace for str {
    fn trim_whitespace(&self) -> &Self {
        self.trim_matches(is_fortran_whitespace)
    }

    fn trim_whitespace_start(&self) -> &Self {
        self.trim_start_matches(is_fortran_whitespace)
    }

    fn trim_whitespace_end(&self) -> &Self {
        self.trim_end_matches(is_fortran_whitespace)
    }
}
