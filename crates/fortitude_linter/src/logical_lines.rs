use fortitude_macros::{kind, kw};
use fortitude_sitter::{Node, traits::TextRanged};
use ruff_source_file::{OneIndexed, SourceCode};
use ruff_text_size::{TextRange, TextSize};

const POST_INDENTORS: [u16; 24] = [
    kind!("program_statement"),
    kind!("module_statement"),
    kind!("submodule_statement"),
    kind!("subroutine_statement"),
    kind!("function_statement"),
    kind!("function"),
    kind!("derived_type_statement"),
    kind!("block_construct"),
    kind!("if_statement"),
    kind!("interface_statement"),
    kind!("procedure_qualifier"),
    kind!("select_case_statement"),
    kind!("select_type_statement"),
    kind!("select_rank_statement"),
    kind!("do_statement"),
    kind!("associate_statement"),
    kind!("where_statement"),
    kind!("contains_statement"),
    kind!("case_statement"),
    kind!("type_statement"),
    kind!("rank_statement"),
    kind!("else_clause"),
    kind!("elseif_clause"),
    kind!("elsewhere_clause"),
];

const DEDENTORS: [u16; 20] = [
    kind!("end_program_statement"),
    kind!("end_module_statement"),
    kind!("end_submodule_statement"),
    kind!("end_subroutine_statement"),
    kind!("end_function_statement"),
    kind!("end_type_statement"),
    kind!("end_block_construct_statement"),
    kind!("end_if_statement"),
    kind!("end_interface_statement"),
    kind!("end_select_statement"),
    kind!("end_do_loop_statement"),
    kind!("end_associate_statement"),
    kind!("end_where_statement"),
    kind!("contains_statement"),
    kind!("case_statement"),
    kind!("type_statement"),
    kind!("rank_statement"),
    kind!("else_clause"),
    kind!("elseif_clause"),
    kind!("elsewhere_clause"),
];

#[allow(dead_code)]
enum LogicalLineEnding {
    Newline,
    Semicolon,
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
        // If the last logical line has not been added, add it now
        if self.current_start_byte != self.current_end_byte {
            let line = &self.source
                [usize::from(self.current_start_byte)..usize::from(self.current_end_byte)];
            let range = TextRange::new(self.current_start_byte, self.current_end_byte);
            let ending = LogicalLineEnding::Newline;
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
            let last_line = LogicalLine {
                line,
                range,
                ending,
                expected_indent,
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
        self.current_start_byte = source.line_start(line_number);
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
            let preproc_continued = self.source[..usize::from(start_byte)]
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
                if self.current_start_byte != self.current_end_byte {
                    let line = &self.source
                        [usize::from(self.current_start_byte)..usize::from(self.current_end_byte)];
                    let range = TextRange::new(self.current_start_byte, self.current_end_byte);
                    let ending = LogicalLineEnding::Newline;
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
                        line,
                        range,
                        ending,
                        expected_indent,
                    };
                    self.lines.push(logical_line);
                }
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
                line: &self.source
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
        if DEDENTORS.contains(&kind_id) {
            self.current_expected_indent = self.current_expected_indent.saturating_sub(1);
            self.next_expected_indent = self.next_expected_indent.saturating_sub(1);
        }
        if POST_INDENTORS.contains(&kind_id) {
            // Edge case: one-line if statements and where statements
            let one_line_if = kind_id == kind!("if_statement")
                && node.child_with_id(kind!("end_if_statement")).is_none();
            let one_line_where = kind_id == kind!("where_statement")
                && node.child_with_id(kind!("end_where_statement")).is_none();
            if !one_line_if && !one_line_where {
                self.next_expected_indent = self.current_expected_indent.saturating_add(1);
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

        assert_eq!(logical_lines.inner.len(), 15);
        let lines = logical_lines.iter().collect_vec();
        assert_eq!(lines[0].line, "module test");
        assert_eq!(lines[1].line, "  implicit none");
        assert_eq!(lines[2].line, "contains");
        assert_eq!(lines[3].line, "  subroutine sub(i)");
        assert_eq!(lines[4].line, "    integer, intent(in) :: i");
        assert_eq!(lines[5].line, "    if (i > 0) then");
        assert_eq!(lines[6].line, "      print *, \"Positive\"");
        assert_eq!(lines[7].line, "    else if (i < 0) then");
        assert_eq!(lines[8].line, "      print *, \"Negative\"");
        assert_eq!(lines[9].line, "    else");
        assert_eq!(lines[10].line, "      print *, \"Zero\"");
        assert_eq!(lines[11].line, "    end if");
        assert_eq!(lines[12].line, "    if (i == 0) print *, \"Still zero\"");
        assert_eq!(lines[13].line, "  end subroutine sub");
        assert_eq!(lines[14].line, "end module test");
        assert_eq!(lines[0].expected_indent, 0);
        assert_eq!(lines[1].expected_indent, 1);
        assert_eq!(lines[2].expected_indent, 0);
        assert_eq!(lines[3].expected_indent, 1);
        assert_eq!(lines[4].expected_indent, 2);
        assert_eq!(lines[5].expected_indent, 2);
        assert_eq!(lines[6].expected_indent, 3);
        assert_eq!(lines[7].expected_indent, 2);
        assert_eq!(lines[8].expected_indent, 3);
        assert_eq!(lines[9].expected_indent, 2);
        assert_eq!(lines[10].expected_indent, 3);
        assert_eq!(lines[11].expected_indent, 2);
        assert_eq!(lines[12].expected_indent, 2);
        assert_eq!(lines[13].expected_indent, 1);
        assert_eq!(lines[14].expected_indent, 0);
        for line in lines {
            assert!(matches!(line.ending, LogicalLineEnding::Newline));
        }
        Ok(())
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

        assert_eq!(logical_lines.inner.len(), 6);
        let lines = logical_lines.iter().collect_vec();
        assert_eq!(lines[0].line, "function f()");
        assert_eq!(lines[1].line, "  do 10 i = 1, n");
        assert_eq!(lines[2].line, "    do 10 j = 1, m");
        assert_eq!(lines[3].line, "      print *, i, j");
        assert_eq!(lines[4].line, "  10 continue");
        assert_eq!(lines[5].line, "end function f");
        for line in lines {
            assert!(matches!(line.ending, LogicalLineEnding::Newline));
        }
        Ok(())
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
end &
& function &
f
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
        assert_eq!(lines[0].line, "function &\n    f()");
        assert_eq!(
            lines[1].line,
            "    print *, &\n\"Hello &\n! mid string comment\n       & World\""
        );
        assert_eq!(lines[2].line, "end &\n& function &\nf");
        assert_eq!(lines[0].expected_indent, 0);
        assert_eq!(lines[1].expected_indent, 1);
        assert_eq!(lines[2].expected_indent, 0);
        for line in lines {
            assert!(matches!(line.ending, LogicalLineEnding::Newline));
        }
        Ok(())
    }

    #[test]
    fn test_semicolons() -> Result<()> {
        let source = r#"
function f();
; ;  print *, "Hello"  ;;print *, "World"
end function f
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

        assert_eq!(logical_lines.inner.len(), 7);
        let lines = logical_lines.iter().collect_vec();
        assert_eq!(lines[0].line, "function f();");
        assert_eq!(lines[1].line, ";"); // First superfluous semicolon on second line
        assert_eq!(lines[2].line, " ;"); // Second superfluous semicolon on second line
        assert_eq!(lines[3].line, "  print *, \"Hello\"  ;");
        assert_eq!(lines[4].line, ";"); // Third superfluous semicolon on second line
        assert_eq!(lines[5].line, "print *, \"World\"");
        assert_eq!(lines[6].line, "end function f");
        assert!(matches!(lines[0].ending, LogicalLineEnding::Semicolon));
        assert!(matches!(lines[1].ending, LogicalLineEnding::Semicolon));
        assert!(matches!(lines[2].ending, LogicalLineEnding::Semicolon));
        assert!(matches!(lines[3].ending, LogicalLineEnding::Semicolon));
        assert!(matches!(lines[4].ending, LogicalLineEnding::Semicolon));
        assert!(matches!(lines[5].ending, LogicalLineEnding::Newline));
        assert!(matches!(lines[6].ending, LogicalLineEnding::Newline));
        Ok(())
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
#undef FOO
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

        assert_eq!(logical_lines.inner.len(), 11);
        let lines = logical_lines.iter().collect_vec();
        assert_eq!(lines[0].line, "#define FOO 1");
        assert_eq!(lines[1].line, "function f()");
        assert_eq!(lines[2].line, "#if FOO \\\n    == 1\n");
        assert_eq!(lines[3].line, "    print *, \"Foo\"");
        assert_eq!(lines[4].line, "#elif FOO == 2\n");
        assert_eq!(lines[5].line, "    print *, \"Bar\"");
        assert_eq!(lines[6].line, "#else");
        assert_eq!(lines[7].line, "    print *, \"Baz\"");
        assert_eq!(lines[8].line, "#endif");
        assert_eq!(lines[9].line, "end function f");
        assert_eq!(lines[10].line, "#undef FOO");
        assert_eq!(lines[0].expected_indent, 0);
        assert_eq!(lines[1].expected_indent, 0);
        assert_eq!(lines[2].expected_indent, 0);
        assert_eq!(lines[3].expected_indent, 1);
        assert_eq!(lines[4].expected_indent, 0);
        assert_eq!(lines[5].expected_indent, 1);
        assert_eq!(lines[6].expected_indent, 0);
        assert_eq!(lines[7].expected_indent, 1);
        assert_eq!(lines[8].expected_indent, 0);
        assert_eq!(lines[9].expected_indent, 0);
        assert_eq!(lines[10].expected_indent, 0);
        for line in lines {
            assert!(matches!(line.ending, LogicalLineEnding::Newline));
        }
        Ok(())
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

        assert_eq!(logical_lines.inner.len(), 11);
        assert_eq!(logical_lines.inner[0].line, "!  line1");
        assert_eq!(logical_lines.inner[1].line, "module mod;");
        assert_eq!(logical_lines.inner[2].line, " implicit &\nnone;");
        assert_eq!(logical_lines.inner[3].line, "    contains ! comment");
        assert_eq!(logical_lines.inner[4].line, "function func() result(res)");
        assert_eq!(logical_lines.inner[5].line, "    integer :: res");
        assert_eq!(logical_lines.inner[6].line, "#if FOO \\\n    == 1\n");
        assert_eq!(logical_lines.inner[7].line, "    res = 1");
        assert_eq!(logical_lines.inner[8].line, "#endif");
        assert_eq!(logical_lines.inner[9].line, "end &\n  & function func;");
        assert_eq!(logical_lines.inner[10].line, "end module mod");
        assert_eq!(logical_lines.inner[0].expected_indent, 0);
        assert_eq!(logical_lines.inner[1].expected_indent, 0);
        assert_eq!(logical_lines.inner[2].expected_indent, 1);
        assert_eq!(logical_lines.inner[3].expected_indent, 0);
        assert_eq!(logical_lines.inner[4].expected_indent, 1);
        assert_eq!(logical_lines.inner[5].expected_indent, 2);
        assert_eq!(logical_lines.inner[6].expected_indent, 0);
        assert_eq!(logical_lines.inner[7].expected_indent, 2);
        assert_eq!(logical_lines.inner[8].expected_indent, 0);
        assert_eq!(logical_lines.inner[9].expected_indent, 1);
        assert_eq!(logical_lines.inner[10].expected_indent, 0);
        assert!(matches!(
            logical_lines.inner[0].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[1].ending,
            LogicalLineEnding::Semicolon
        ));
        assert!(matches!(
            logical_lines.inner[2].ending,
            LogicalLineEnding::Semicolon
        ));
        assert!(matches!(
            logical_lines.inner[3].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[4].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[5].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[6].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[7].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[8].ending,
            LogicalLineEnding::Newline
        ));
        assert!(matches!(
            logical_lines.inner[9].ending,
            LogicalLineEnding::Semicolon
        ));
        assert!(matches!(
            logical_lines.inner[10].ending,
            LogicalLineEnding::Newline
        ));

        Ok(())
    }
}
