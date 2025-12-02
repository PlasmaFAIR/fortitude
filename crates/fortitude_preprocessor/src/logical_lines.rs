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

    /// Merge two logical lines at the given end and start indices, returning
    /// a new LogicalLine.
    pub fn merge(
        self,
        right: LogicalLine,
        left_end: TextSize,
        right_start: TextSize,
    ) -> LogicalLine<'a> {
        let merged_text = {
            let left_part = &self.text[..left_end.to_usize()];
            let right_part = &right.text[right_start.to_usize()..];
            left_part.to_owned() + right_part
        };
        let merged_byte_map = {
            let mut map = self.byte_map.clone();
            // Remove any entries in left.byte_map after left_end
            map.retain(|&k, _| k < left_end);
            // Add beginning entry for right_start
            map.insert(left_end, right.offset(right_start));
            // Add entries from right.byte_map after right_start, adjusted for offset
            for (src, dest) in right.byte_map.iter().filter(|(k, _)| *k > &right_start) {
                map.insert(left_end + src - right_start, *dest);
            }
            map
        };
        LogicalLine {
            text: Cow::Owned(merged_text),
            byte_map: merged_byte_map,
        }
    }
}

pub struct LogicalLines<'a> {
    lines: Vec<LogicalLine<'a>>,
}

impl<'a> LogicalLines<'a> {
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
                        let left_end = TextSize::from(continued.len() as u32);
                        let right_start = TextSize::from(0);
                        let merged = prev_line.merge(line, left_end, right_start);
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

    pub fn from_source_code(src: &'a SourceCode) -> LogicalLines<'a> {
        // TODO implement merging of multiline comments
        LogicalLines::from_source_code_unmerged(src).merge_escaped_newlines()
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
}
