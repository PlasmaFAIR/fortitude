use crate::tokens::{CppTokenIterator, CppTokenKind};
use ruff_source_file::{OneIndexed, SourceCode};
use ruff_text_size::TextSize;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// A logical line of code, which may span multiple physical lines due to
/// escaped newlines and C-style comments. Tracks the byte offset of each
/// location of the logical line.
#[derive(Clone)]
pub struct LogicalLine<'a> {
    /// The text of the logical line.
    text: Cow<'a, str>,
    /// The byte offsets of each character in the logical line
    /// relative to the start of the source file.
    byte_map: BTreeMap<TextSize, TextSize>,
}

impl<'a> LogicalLine<'a> {
    /// Create a new LogicalLine from source code. Includes only the requested
    /// line, and borrows the text from the source code. To handle escaped
    /// newlines and comments, use the `merge` method.
    pub fn from_source_code(src: &'a SourceCode, line: OneIndexed) -> LogicalLine<'a> {
        LogicalLine {
            text: Cow::Borrowed(src.line_text(line)),
            byte_map: BTreeMap::from([(0.into(), src.line_start(line))]),
        }
    }

    /// Reference to the text of the logical line. Hides the internal Cow.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the byte offset in the source file corresponding to the given
    /// index in the logical line. Returns None if the index is out of bounds.
    pub fn offset(&self, index: TextSize) -> TextSize {
        // Can safely unwrap because the byte_map always contains a mapping for 0
        let (src, dest) = self.byte_map.range(..=index).next_back().unwrap();
        dest + (index - src)
    }

    /// Get the byte offset range spanned by this logical line.
    pub fn offset_range(&self) -> (TextSize, TextSize) {
        (
            self.offset(0.into()),
            self.offset(TextSize::from(self.text.len() as u32)),
        )
    }

    /// Merge two logical lines at the end index of `self`, returning
    /// a new LogicalLine.
    fn merge(self, other: LogicalLine, end_index: TextSize) -> LogicalLine<'a> {
        let merged_text = self.text()[..end_index.to_usize()].to_owned() + other.text();
        let merged_byte_map = {
            let mut map = self.byte_map.clone();
            // Remove any entries in left.byte_map after left_end
            map.retain(|&k, _| k < end_index);
            // Add entries from right.byte_map
            for (src, dest) in other.byte_map {
                map.insert(src + end_index, dest);
            }
            map
        };
        LogicalLine {
            text: Cow::Owned(merged_text),
            byte_map: merged_byte_map,
        }
    }

    /// Determine local byte offset of the start of an unterminated comment.
    /// The byte returned is the first byte within the comment itself.
    /// Returns None if no unterminated comment is found.
    fn unterminated_comment_start(&self) -> Option<TextSize> {
        // Tokenise the line to avoid comment starts inside string literals.
        // While the last token should ordinarily be a newline, if the line
        // ends with an unterminated comment, the newline will be contained
        // within the comment token.
        let last_token = CppTokenIterator::new(self.text()).last()?;
        if last_token.kind == CppTokenKind::Comment {
            Some(last_token.start + TextSize::from(2)) // +2 to get past the /*
        } else {
            None
        }
    }
}

pub struct LogicalLines<'a> {
    lines: Vec<LogicalLine<'a>>,
}

impl<'a> LogicalLines<'a> {
    /// Generate unmerged logical lines from source code.
    /// Each line corresponds directly to a physical line in the source code,
    /// and borrows the text from the source code.
    fn from_source_code_unmerged(src: &'a SourceCode) -> LogicalLines<'a> {
        let mut lines = Vec::new();
        let line_count = src.line_count();
        for i in 0..line_count {
            lines.push(LogicalLine::from_source_code(
                src,
                OneIndexed::from_zero_indexed(i),
            ));
        }
        LogicalLines { lines }
    }

    /// Iterate over all logical lines, merging those that are continued
    /// with a trailing backslash.
    fn merge_escaped_newlines(self) -> Self {
        let mut new = Vec::new();
        let mut prev: Option<LogicalLine<'_>> = None;
        for line in self.lines.into_iter() {
            match prev {
                Some(prev_line) => {
                    let trimmed = prev_line.text().trim_end();
                    if let Some(continued) = trimmed.strip_suffix('\\') {
                        // If the previous line ends with a backslash, merge it
                        // with this line
                        let end_index = TextSize::from(continued.len() as u32);
                        let merged = prev_line.merge(line, end_index);
                        prev = Some(merged);
                    } else {
                        // If not, insert the previous line and set this line as
                        // the new previous.
                        new.push(prev_line);
                        prev = Some(line);
                    }
                }
                None => {
                    prev = Some(line);
                }
            }
        }
        // Insert the last line if it exists
        if let Some(last_line) = prev {
            new.push(last_line);
        }
        LogicalLines { lines: new }
    }

    /// Iterate over all logical lines, merging those linked by a multi-line
    /// C-style comment. The comment in the merged line will contain only the
    /// contents on the last line up to the terminator.
    fn merge_multiline_comments(self) -> Self {
        let mut new = Vec::new();
        let mut unterminated: Option<(LogicalLine<'_>, TextSize)> = None;
        for line in self.lines.into_iter() {
            match unterminated {
                Some((prev_line, end_index)) => {
                    // If we have an unterminated comment, merge it with this line
                    let merged = prev_line.merge(line, end_index);
                    // If the merged line still has an unterminated comment,
                    // store it. Otherwise, insert the merged line and clear the
                    // unterminated state.
                    let new_idx = merged.unterminated_comment_start();
                    if let Some(new_idx) = new_idx {
                        unterminated = Some((merged, new_idx));
                    } else {
                        new.push(merged);
                        unterminated = None;
                    }
                }
                None => {
                    let idx = line.unterminated_comment_start();
                    if let Some(idx) = idx {
                        // If the line contains an unterminated comment, store it
                        unterminated = Some((line, idx));
                    } else {
                        // Otherwise, insert the line directly
                        new.push(line);
                    }
                }
            }
        }
        // Insert the last line if it exists
        if let Some((last_line, _)) = unterminated {
            new.push(last_line);
        }
        LogicalLines { lines: new }
    }

    pub fn from_source_code(src: &'a SourceCode) -> LogicalLines<'a> {
        LogicalLines::from_source_code_unmerged(src)
            .merge_escaped_newlines()
            .merge_multiline_comments()
    }
}

impl<'a> IntoIterator for LogicalLines<'a> {
    type Item = LogicalLine<'a>;
    type IntoIter = std::vec::IntoIter<LogicalLine<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;
    use ruff_source_file::LineIndex;

    #[test]
    fn test_logical_lines_escaped_newlines() {
        let code = dedent!(
            r#"
            program\
            \
            \  
             p
              im\   
            plicit none
            end program \
            p
            "#
        );
        let line_index = LineIndex::from_source_text(code);
        let source_code = SourceCode::new(code, &line_index);
        let lines = LogicalLines::from_source_code(&source_code)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text(), "program p\n");
        assert_eq!(lines[1].text(), "  implicit none\n");
        assert_eq!(lines[2].text(), "end program p");
    }

    #[test]
    fn test_logical_lines_multiline_comments() {
        let code = dedent!(
            r#"
            program p
              print *, /**/ "/* hello /*" /*
              * 
              " */ /* */ /*
              *
              */, "/* world */"
            end program p
            "#
        );
        let line_index = LineIndex::from_source_text(code);
        let source_code = SourceCode::new(code, &line_index);
        let lines = LogicalLines::from_source_code(&source_code)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text(), "program p\n");
        assert_eq!(
            lines[1].text(),
            "  print *, /**/ \"/* hello /*\" /*  \" */ /* */ /*  */, \"/* world */\"\n"
        );
        assert_eq!(lines[2].text(), "end program p");
    }

    #[test]
    fn test_logical_lines_multiline_comments_unterminated_string() {
        let code = dedent!(
            r#"
            program p
              print *, " hello /*
              */ world "
            end program p
            "#
        );
        let line_index = LineIndex::from_source_text(code);
        let source_code = SourceCode::new(code, &line_index);
        let transformed = LogicalLines::from_source_code(&source_code)
            .into_iter()
            .map(|line| line.text().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(transformed, code);
    }

    #[test]
    fn test_logical_lines_multiline_comments_and_escaped_newlines() {
        // Should include a few more characters than before
        let code = dedent!(
            r#"
            program p
              print *, /**/ "/* hello /*" /*
              *\ 
              " */ /* */ /*
              *\
              */, "/* world */"
            end program p
            "#
        );
        let line_index = LineIndex::from_source_text(code);
        let source_code = SourceCode::new(code, &line_index);
        let lines = LogicalLines::from_source_code(&source_code)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text(), "program p\n");
        assert_eq!(
            lines[1].text(),
            "  print *, /**/ \"/* hello /*\" /*  *  \" */ /* */ /*  *  */, \"/* world */\"\n"
        );
        assert_eq!(lines[2].text(), "end program p");
    }
}
