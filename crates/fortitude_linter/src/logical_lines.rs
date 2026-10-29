use crate::indentation::{DEDENTORS, IndentMap, POST_INDENTORS};
use fortitude_macros::{kind, kw};
use fortitude_sitter::{Node, traits::TextRanged};
use ruff_source_file::{OneIndexed, SourceCode};
use ruff_text_size::{TextRange, TextSize};

#[derive(Debug, PartialEq)]
pub(crate) enum LogicalLineEnding {
    Newline,
    Semicolon,
}

pub(crate) struct LogicalLine<'source> {
    text: &'source str,
    range: TextRange,
    expected_indent: u8,
    #[allow(dead_code)]
    ending: LogicalLineEnding,
}

impl<'source> LogicalLine<'source> {
    pub fn text(&self) -> &'source str {
        self.text
    }

    pub fn start_byte(&self) -> TextSize {
        self.range.start()
    }

    #[allow(dead_code)]
    pub fn end_byte(&self) -> TextSize {
        self.range.end()
    }

    pub fn expected_indent(&self) -> u8 {
        self.expected_indent
    }

    /// Leading whitespace, plus any statement label and the whitespace after it.
    pub fn indentation(&self) -> &'source str {
        let leading = self.leading_whitespace_len();
        let mut count = leading;
        if let Some(label) = self.label() {
            count += label.len();
            count += self.text[count..]
                .chars()
                .take_while(|c| c.is_ascii_whitespace())
                .count();
        }
        &self.text[..count]
    }

    /// The numeric statement label (e.g. `10` in `  10 continue`), if present.
    pub fn label(&self) -> Option<&'source str> {
        let rest = &self.text[self.leading_whitespace_len()..];
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        (digits > 0 && rest[digits..].starts_with(|c: char| c.is_ascii_whitespace()))
            .then(|| &rest[..digits])
    }

    fn leading_whitespace_len(&self) -> usize {
        self.text
            .chars()
            .take_while(|c| c.is_ascii_whitespace())
            .count()
    }

    /// The byte range of the indentation
    pub fn indentation_range(&self) -> TextRange {
        TextRange::new(
            self.start_byte(),
            self.start_byte() + TextSize::from(self.indentation().len() as u32),
        )
    }

    #[allow(dead_code)]
    pub fn ending(&self) -> &LogicalLineEnding {
        &self.ending
    }
}

pub(crate) struct LogicalLines<'source> {
    inner: Vec<LogicalLine<'source>>,
}

impl<'source> LogicalLines<'source> {
    /// Returns an iterator over the logical lines
    pub fn iter(&self) -> impl Iterator<Item = &LogicalLine<'source>> {
        self.inner.iter()
    }
}

/// A builder for constructing `LogicalLines` from a source code string.
/// The build occurs incrementally as the tree-sitter parse tree is traversed.
/// The builder maintains the current state of the logical line being
/// constructed, including the start position, expected indentation, and the
/// list of completed logical lines. When all nodes have been processed, the
/// `finish` method is called to finalise the construction and return the
/// `LogicalLines` object.
pub(crate) struct LogicalLinesBuilder<'source> {
    source: &'source str,
    lines: Vec<LogicalLine<'source>>,
    indent_map: IndentMap,
    current_line_number: OneIndexed,
    current_start_byte: TextSize,
    current_end_byte: TextSize,
    current_expected_indent: u8,
    next_expected_indent: u8,
    indent_stack: Vec<u8>,
}

impl<'source> LogicalLinesBuilder<'source> {
    /// Create a new builder for the given source code.
    pub fn new(root: Node<'source>, source: &'source SourceCode, indent_map: IndentMap) -> Self {
        let start_byte = root.start_textsize();
        let line_number = source.line_index(start_byte);
        Self {
            source: source.text(),
            lines: Vec::new(),
            indent_map,
            current_line_number: line_number,
            current_start_byte: source.line_start(line_number),
            current_end_byte: start_byte,
            current_expected_indent: 0,
            next_expected_indent: 0,
            indent_stack: vec![0],
        }
    }

    /// Consume the builder and returns the constructed `LogicalLines`.
    pub fn finish(mut self) -> LogicalLines<'source> {
        // If the last logical line has not been added, add it now
        self.finish_line();
        LogicalLines { inner: self.lines }
    }

    fn finish_line(&mut self) {
        if self.current_start_byte == self.current_end_byte {
            return;
        }

        let line =
            &self.source[usize::from(self.current_start_byte)..usize::from(self.current_end_byte)];
        let range = TextRange::new(self.current_start_byte, self.current_end_byte);
        // If the first non-whitespace character on the line is a '#',
        // it is a preprocessor directive, so the expected indentation
        // is zero.
        let first_non_whitespace = line
            .chars()
            .find(|c| !c.is_ascii_whitespace())
            .unwrap_or('\0');
        let expected_indent = if first_non_whitespace == '#' {
            0
        } else {
            self.current_expected_indent
        };
        let logical_line = LogicalLine {
            text: line,
            range,
            ending: LogicalLineEnding::Newline,
            expected_indent,
        };
        self.lines.push(logical_line);
    }

    pub fn add_node(&mut self, node: Node<'source>, source: &'source SourceCode) {
        let start_byte = node.start_textsize();
        let end_byte = node.end_textsize();
        let line_number = source.line_index(start_byte);
        let kind_id = node.kind_id();

        // If the node starts on a new line, we might write out the previous line.
        if line_number > self.current_line_number {
            // Continued lines begin with a real '&' node or a virtual one of
            // zero width located at the beginning of the first node on the
            // line. If this is found on a new line, the current logical line is
            // extended to include the next line.
            // We can also check for preprocessor line continuations by checking
            // if the current line ends with a backslash.
            let continued = node.kind_id() == kw!("&");
            // As line number must be greater than current_line_number, we can
            // safely subtract 1 to get the previous line.
            let last_physical_line = source.line_text(line_number.saturating_sub(1));
            let preproc_continued = last_physical_line
                .chars()
                .rev()
                .find(|c| !c.is_ascii_whitespace())
                == Some('\\');
            if !continued && !preproc_continued {
                // The node starts on a new line, so we need to finish the
                // current logical line.
                // It is possible for the current logical line to be empty if
                // the previous line ended with a semicolon, so we check that
                // the start and end bytes are not equal before adding the line.
                self.finish_line();
                self.current_start_byte = source.line_start(line_number);
                self.current_expected_indent = self.next_expected_indent;
            }
            self.current_line_number = line_number;
        }

        // If the node is a semicolon, we need to finish the current logical
        // line and start a new one. Note that this can happen at the very
        // start of a new line, so it must be checked after the new line check
        // above.
        // Semicolons can also really mess with indentation! Here, we assume
        // that the indentation of the next logical line is the same as if the
        // semicolon were not there, otherwise it's easy to find yourself in
        // a situation where the indentation is off by one for the rest of
        // the file.
        if kind_id == kw!(";") {
            // End byte is extended immediately to include the semicolon
            self.current_end_byte = end_byte;
            let logical_line = LogicalLine {
                text: &self.source
                    [usize::from(self.current_start_byte)..usize::from(self.current_end_byte)],
                range: TextRange::new(self.current_start_byte, self.current_end_byte),
                ending: LogicalLineEnding::Semicolon,
                expected_indent: self.current_expected_indent,
            };
            self.lines.push(logical_line);
            self.current_start_byte = end_byte;
            self.current_expected_indent = self.next_expected_indent;
        }

        // Dedentors and post-indentors affect the expected indentation of the
        // next logical line, so this must be performed after the current
        // logical line has been finalised.
        let last_indent = self.indent_stack.last().copied().unwrap_or(0);
        if DEDENTORS.contains(&kind_id) {
            let dedent = self.indent_stack.pop().unwrap_or(0);
            self.current_expected_indent = self.current_expected_indent.saturating_sub(dedent);
            self.next_expected_indent = self.next_expected_indent.saturating_sub(dedent);
        }
        if POST_INDENTORS.contains(&kind_id) {
            // Edge case: one-line if statements and where statements
            let one_line_if = kind_id == kind!("if_statement")
                && node.child_with_id(kind!("end_if_statement")).is_none();
            let one_line_where = kind_id == kind!("where_statement")
                && node.child_with_id(kind!("end_where_statement")).is_none();
            if !one_line_if && !one_line_where {
                let indent = if kind_id == kind!("contains_statement") {
                    // The indent is determined by the context in which the
                    // contains statement appears.
                    last_indent
                } else {
                    self.indent_map.get_indent(&kind_id)
                };
                self.next_expected_indent = self.current_expected_indent.saturating_add(indent);
                self.indent_stack.push(indent);
            }
        }

        // Regardless of what happened, the current logical line is extended.
        // In the case of finding a semicolon, this action is repeated twice,
        // but this is harmless.
        self.current_end_byte = end_byte;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use fortitude_sitter::Parser;
    use itertools::Itertools;
    use ruff_source_file::LineIndex;

    macro_rules! assert_logical_line {
        ($line:expr, $text:expr, $expected_indent:expr) => {
            assert_eq!($line.text, $text);
            assert_eq!($line.expected_indent, $expected_indent);
        };
        ($line:expr, $text:expr, $expected_indent:expr, Newline) => {
            assert_eq!($line.text, $text);
            assert_eq!($line.expected_indent, $expected_indent);
            assert_eq!($line.ending, LogicalLineEnding::Newline);
        };
        ($line:expr, $text:expr, $expected_indent:expr, Semicolon) => {
            assert_eq!($line.text, $text);
            assert_eq!($line.expected_indent, $expected_indent);
            assert_eq!($line.ending, LogicalLineEnding::Semicolon);
        };
    }

    fn test_logical_lines<F>(source: &str, test: F) -> Result<()>
    where
        F: FnOnce(&[LogicalLine<'_>]) -> Result<()>,
    {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;
        let tree = parser.parse(source, None).context("Failed to parse")?;
        let root = tree.root_node();
        let line_index = LineIndex::from_source_text(source);
        let source_code = SourceCode::new(source, &line_index);
        let indent_map = IndentMap::default();
        let mut builder = LogicalLinesBuilder::new(root, &source_code, indent_map);
        for node in root.descendants() {
            builder.add_node(node, &source_code);
        }
        let logical_lines = builder.finish();
        test(&logical_lines.inner)
    }

    #[test]
    fn test_simple_lines() -> Result<()> {
        let source = r#"
! Empty program
program test

end program test
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 3);
            assert_logical_line!(lines[0], "! Empty program", 0);
            assert_logical_line!(lines[1], "program test", 0);
            assert_logical_line!(lines[2], "end program test", 0);
            Ok(())
        })
    }

    #[test]
    fn test_indents() -> Result<()> {
        let source = r#"
module test
    implicit none
contains
    subroutine sub(i)
        integer, intent(in) :: i
        if (i > 0) then
            print *, "Positive"
        else if (i < 0) then
            print *, "Negative"
        else
            print *, "Zero"
        end if
        if (i == 0) print *, "Still zero"
    end subroutine sub
end module test
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 15);
            assert_logical_line!(lines[0], "module test", 0);
            assert_logical_line!(lines[1], "    implicit none", 4);
            assert_logical_line!(lines[2], "contains", 0);
            assert_logical_line!(lines[3], "    subroutine sub(i)", 4);
            assert_logical_line!(lines[4], "        integer, intent(in) :: i", 8);
            assert_logical_line!(lines[5], "        if (i > 0) then", 8);
            assert_logical_line!(lines[6], "            print *, \"Positive\"", 12);
            assert_logical_line!(lines[7], "        else if (i < 0) then", 8);
            assert_logical_line!(lines[8], "            print *, \"Negative\"", 12);
            assert_logical_line!(lines[9], "        else", 8);
            assert_logical_line!(lines[10], "            print *, \"Zero\"", 12);
            assert_logical_line!(lines[11], "        end if", 8);
            assert_logical_line!(lines[12], "        if (i == 0) print *, \"Still zero\"", 8);
            assert_logical_line!(lines[13], "    end subroutine sub", 4);
            assert_logical_line!(lines[14], "end module test", 0);
            Ok(())
        })
    }

    /// Test that a nested non-block do loop terminated by `continue` is handled
    /// correctly. This should work, as the tree-sitter grammar defines a 'virtual'
    /// end_do_loop_statement node for this case, which is a dedentor.
    #[test]
    fn test_do_continue() -> Result<()> {
        let source = r#"
function f()
    do 10 i = 1, n
        do 10 j = 1, m
            print *, i, j
    10 continue
end function f
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 6);
            assert_logical_line!(lines[0], "function f()", 0, Newline);
            assert_logical_line!(lines[1], "    do 10 i = 1, n", 4, Newline);
            assert_logical_line!(lines[2], "        do 10 j = 1, m", 8, Newline);
            assert_logical_line!(lines[3], "            print *, i, j", 12, Newline);
            assert_logical_line!(lines[4], "    10 continue", 4, Newline);
            assert_logical_line!(lines[5], "end function f", 0, Newline);
            Ok(())
        })
    }

    #[test]
    fn test_line_continuations() -> Result<()> {
        let source = r#"
function &
    f()
    print *, &
"Hello &
! mid string comment
       & World"

end \
function &
f
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 4);
            assert_logical_line!(lines[0], "function &\n    f()", 0, Newline);
            assert_logical_line!(lines[1], "    print *, &\n\"Hello &", 4, Newline);
            assert_logical_line!(
                lines[2],
                "! mid string comment\n       & World\"",
                4,
                Newline
            );
            assert_logical_line!(lines[3], "end \\\nfunction &\nf", 0, Newline);
            Ok(())
        })
    }

    #[test]
    fn test_semicolons() -> Result<()> {
        let source = r#"
function f();
; ;  print *, "Hello"  ;;print *, "World"
end function f
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 7);
            // Note: the expected indentations and the actual indentations do not match!
            // All lines inside the function should be indented by 4 spaces, regardless of
            // whether or not they are preceded by semicolons.
            assert_logical_line!(lines[0], "function f();", 0, Semicolon);
            assert_logical_line!(lines[1], ";", 4, Semicolon); // First superfluous semicolon on second line
            assert_logical_line!(lines[2], " ;", 4, Semicolon); // Second superfluous semicolon on second line
            assert_logical_line!(lines[3], "  print *, \"Hello\"  ;", 4, Semicolon);
            assert_logical_line!(lines[4], ";", 4, Semicolon); // Third superfluous semicolon on second line
            assert_logical_line!(lines[5], "print *, \"World\"", 4, Newline);
            assert_logical_line!(lines[6], "end function f", 0, Newline);
            Ok(())
        })
    }

    #[test]
    fn test_preprocessor() -> Result<()> {
        let source = r#"
#define FOO 1
function f()
#if FOO \
    == 1
    print *, "Foo"
#elif FOO == 2
    print *, "Bar"
#else
    print *, "Baz"
#endif
end function f
#undef \
FOO
"#;
        test_logical_lines(source, |lines| {
            assert_eq!(lines.len(), 11);
            assert_logical_line!(lines[0], "#define FOO 1", 0, Newline);
            assert_logical_line!(lines[1], "function f()", 0, Newline);
            assert_logical_line!(lines[2], "#if FOO \\\n    == 1\n", 0, Newline);
            assert_logical_line!(lines[3], "    print *, \"Foo\"", 4, Newline);
            assert_logical_line!(lines[4], "#elif FOO == 2\n", 0, Newline);
            assert_logical_line!(lines[5], "    print *, \"Bar\"", 4, Newline);
            assert_logical_line!(lines[6], "#else", 0, Newline);
            assert_logical_line!(lines[7], "    print *, \"Baz\"", 4, Newline);
            assert_logical_line!(lines[8], "#endif", 0, Newline);
            assert_logical_line!(lines[9], "end function f", 0, Newline);
            assert_logical_line!(lines[10], "#undef \\\nFOO", 0, Newline);
            Ok(())
        })
    }

    #[test]
    fn test_complex() -> Result<()> {
        let source = r#"
!  line1
module mod; implicit &
none;    contains ! comment
function func() result(res)
    integer :: res
#if FOO \
    == 1
    res = 1
#endif
end &
  & function func;end module mod
"#;
        test_logical_lines(source, |logical_lines| {
            let lines = logical_lines.iter().collect_vec();

            assert_eq!(lines.len(), 11);
            assert_logical_line!(lines[0], "!  line1", 0, Newline);
            assert_logical_line!(lines[1], "module mod;", 0, Semicolon);
            assert_logical_line!(lines[2], " implicit &\nnone;", 4, Semicolon);
            assert_logical_line!(lines[3], "    contains ! comment", 0, Newline);
            assert_logical_line!(lines[4], "function func() result(res)", 4, Newline);
            assert_logical_line!(lines[5], "    integer :: res", 8, Newline);
            assert_logical_line!(lines[6], "#if FOO \\\n    == 1\n", 0, Newline);
            assert_logical_line!(lines[7], "    res = 1", 8, Newline);
            assert_logical_line!(lines[8], "#endif", 0, Newline);
            assert_logical_line!(lines[9], "end &\n  & function func;", 4, Semicolon);
            assert_logical_line!(lines[10], "end module mod", 0, Newline);
            Ok(())
        })
    }
}
