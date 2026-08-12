use fortitude_sitter::{Node, traits::TextRanged};
use ruff_source_file::{OneIndexed, SourceCode};
use ruff_text_size::{TextRange, TextSize};

#[allow(dead_code)]
enum LogicalLineEnding {
    Newline,
    Semicolon,
    /// A logical line that is not present in the source code, but is inferred
    /// from the context. For example, a nested non-block do loop terminated by
    /// `continue`.
    Virtual,
}

#[allow(dead_code)]
struct LogicalLine<'source> {
    line: &'source str,
    range: TextRange,
    expected_indent: u8,
    ending: LogicalLineEnding,
}

#[allow(dead_code)]
struct LogicalLines<'source> {
    source: &'source str,
    inner: Vec<LogicalLine<'source>>,
}

impl<'source> LogicalLines<'source> {
    /// Returns an iterator over the logical lines
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &LogicalLine<'source>> {
        self.inner.iter()
    }

    /// Returns the number of lines
    #[inline]
    #[allow(dead_code)]
    pub fn line_count(&self) -> usize {
        self.inner.len()
    }
}

/// A builder for constructing `LogicalLines` from a source code string.
/// The build occurs incrementally as the tree-sitter parse tree is traversed.
/// The builder maintains the current state of the logical line being
/// constructed, including the start position, expected indentation, and the
/// list of completed logical lines. When all nodes have been processed, the
/// `finish` method is called to finalise the construction and return the
/// `LogicalLines` object.
#[allow(dead_code)]
struct LogicalLinesBuilder<'source> {
    source: &'source str,
    lines: Vec<LogicalLine<'source>>,
    current_line_number: OneIndexed,
    current_start_byte: TextSize,
    current_end_byte: TextSize,
    last_line_continued: bool,
    current_expected_indent: u8,
    next_expected_indent: u8,
    initialised: bool,
}

impl<'source> LogicalLinesBuilder<'source> {
    /// Create a new builder for the given source code.
    #[allow(dead_code)]
    fn new(source: &'source str) -> Self {
        Self {
            source,
            lines: Vec::new(),
            current_line_number: OneIndexed::from_zero_indexed(0),
            current_start_byte: TextSize::from(0),
            current_end_byte: TextSize::from(0),
            last_line_continued: false,
            current_expected_indent: 0,
            next_expected_indent: 0,
            initialised: false,
        }
    }

    /// Consume the builder and returns the constructed `LogicalLines`.
    #[allow(dead_code)]
    fn finish(self) -> LogicalLines<'source> {
        let mut lines = self.lines;
        if self.current_start_byte != self.current_end_byte {
            // If the last logical line has not been added, add it now
            let last_line = LogicalLine {
                line: &self.source
                    [usize::from(self.current_start_byte)..usize::from(self.current_end_byte)],
                range: TextRange::new(self.current_start_byte, self.current_end_byte),
                ending: LogicalLineEnding::Newline,
                expected_indent: self.current_expected_indent,
            };
            lines.push(last_line);
        }
        LogicalLines {
            source: self.source,
            inner: lines,
        }
    }

    /// Called by `add_node` on first encountering a node.
    #[allow(dead_code)]
    fn initialise(&mut self, node: Node<'source>, source: &'source SourceCode) {
        let start_byte = node.start_textsize();
        let line_number = source.line_index(start_byte);
        self.current_line_number = line_number;
        self.current_start_byte = start_byte;
        self.current_end_byte = start_byte;
        self.initialised = true;
    }

    #[allow(dead_code)]
    fn add_node(&mut self, node: Node<'source>, source: &'source SourceCode) {
        if !self.initialised {
            self.initialise(node, source);
        }

        let start_byte = node.start_textsize();
        let end_byte = node.end_textsize();
        let line_number = source.line_index(start_byte);
        if line_number > self.current_line_number {
            // The node starts on a new line, so we need to finish the current logical line
            let line = LogicalLine {
                expected_indent: 0,
                line: &self.source
                    [usize::from(self.current_start_byte)..usize::from(self.current_end_byte)],
                range: TextRange::new(self.current_start_byte, self.current_end_byte),
                ending: LogicalLineEnding::Newline,
            };
            self.lines.push(line);
            self.current_line_number = line_number;
            self.current_start_byte = start_byte;
            self.current_end_byte = end_byte;
        } else {
            // The node is on the same line, so we extend the current logical line
            self.current_end_byte = end_byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use fortitude_sitter::Parser;
    use itertools::Itertools;
    use ruff_source_file::LineIndex;

    #[test]
    fn test_simple_lines() -> Result<()> {
        let source = r#"
! Empty program
program test
end program test
"#;
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;
        let tree = parser.parse(source, None).context("Failed to parse")?;
        let root = tree.root_node();
        let line_index = LineIndex::from_source_text(source);
        let source_code = SourceCode::new(source, &line_index);
        let mut builder = LogicalLinesBuilder::new(source);
        for node in root.descendants() {
            builder.add_node(node, &source_code);
        }
        let logical_lines = builder.finish();

        assert_eq!(logical_lines.inner.len(), 3);
        let lines = logical_lines.iter().collect_vec();
        assert_eq!(lines[0].line, "! Empty program");
        assert_eq!(lines[1].line, "program test");
        assert_eq!(lines[2].line, "end program test");
        for line in lines {
            assert!(matches!(line.ending, LogicalLineEnding::Newline));
            assert_eq!(line.expected_indent, 0);
        }
        Ok(())
    }

    //     #[test]
    //     fn test_logical_lines() -> Result<()> {
    //         let code = r#"
    // !  line1
    // module mod; implicit &
    // none;    contains ! comment
    // function func() result(res)
    //     integer :: res
    //     res = 1
    // end &
    //   function func;end module mod
    // "#;
    //         let logical_lines = build_logical_lines(code)?;

    //         assert_eq!(logical_lines.inner.len(), 8);
    //         assert_eq!(logical_lines.inner[0].line, "!  line1\n");
    //         assert_eq!(logical_lines.inner[1].line, " implicit &\nnone;");
    //         assert_eq!(logical_lines.inner[2].line, "    contains ! comment\n");
    //         assert_eq!(logical_lines.inner[3].line, "function func() result(res)\n");
    //         assert_eq!(logical_lines.inner[4].line, "    integer :: res\n");
    //         assert_eq!(logical_lines.inner[5].line, "    res = 1\n");
    //         assert_eq!(logical_lines.inner[6].line, "end &\n  function func;");
    //         assert_eq!(logical_lines.inner[7].line, "end module mod\n");
    //         Ok(())
    //     }
}
