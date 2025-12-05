mod definitions;
mod logical_lines;
mod tokens;

use anyhow::{Context, anyhow};
use chrono::prelude::*;
use definitions::{Definition, Definitions, MacroKind};
use lazy_regex::regex_captures;
use logical_lines::LogicalLines;
use ruff_source_file::SourceCode;
use ruff_text_size::TextSize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokens::{CppDirectiveKind, CppToken, CppTokenIterator, CppTokenKind};

/// Enum describing where a snippet came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Provenance {
    /// A system-defined macro, e.g. __DATE__ or __TIME__.
    SystemDefined,
    /// A user-defined macro, e.g. passed in via command line.
    UserDefined,
    /// A macro defined in the source file.
    FileDefined {
        start: TextSize,
        end: TextSize,
        path: PathBuf,
    },
    /// Plain text from an included file.
    IncludeText { path: PathBuf },
    /// Plain text in the source file, not from a macro.
    LocalText { start: TextSize, end: TextSize },
}

/// A snippet of text with provenance information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snippet {
    text: String,
    provenance: Provenance,
}

impl Snippet {
    pub fn extend(&mut self, other: &Snippet) -> anyhow::Result<()> {
        match (&self.provenance, &other.provenance) {
            (
                Provenance::LocalText {
                    start: self_start,
                    end: self_end,
                },
                Provenance::LocalText {
                    start: other_start,
                    end: other_end,
                },
            ) => {
                if self_end == other_start {
                    self.text.push_str(&other.text);
                    self.provenance = Provenance::LocalText {
                        start: *self_start,
                        end: *other_end,
                    };
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Can only extend snippets with contiguous provenance"
                    ))
                }
            }
            _ => Err(anyhow!(
                "Can only extend snippets with LocalText provenance"
            )),
        }
    }
}

/// A collection of snippets.
pub struct Snippets {
    // Collected snippets
    inner: Vec<Snippet>,
}

impl Default for Snippets {
    fn default() -> Self {
        Self::new()
    }
}

impl Snippets {
    pub fn new() -> Self {
        Snippets { inner: Vec::new() }
    }

    pub fn push(&mut self, text: &str, provenance: Provenance) {
        let snippet = Snippet {
            text: text.to_string(),
            provenance,
        };
        if let Some(last) = self.inner.last_mut() {
            if last.extend(&snippet).is_ok() {
                return;
            }
        }
        self.inner.push(snippet);
    }

    pub fn collect(&self) -> String {
        self.inner
            .iter()
            .map(|s| s.text.as_str())
            .collect::<String>()
    }
}

/// A stack to track conditional compilation state.
pub struct IfStack {
    stack: Vec<bool>,
}

impl Default for IfStack {
    fn default() -> Self {
        Self::new()
    }
}

impl IfStack {
    /// Create a new, empty IfStack.
    pub fn new() -> Self {
        IfStack { stack: Vec::new() }
    }

    /// Push a new conditional state onto the stack. If
    /// the stack is already false, push false.
    pub fn push(&mut self, state: bool) {
        if self.is_clean() {
            self.stack.push(state);
        } else {
            self.stack.push(false);
        }
    }

    /// Pop the top conditional state from the stack.
    pub fn pop(&mut self) -> Option<bool> {
        self.stack.pop()
    }

    /// Toggle the top conditional state on the stack.
    /// If the stack has more than one false, does nothing.
    pub fn toggle(&mut self) -> Option<bool> {
        match self.pop() {
            Some(state) => {
                // If there are at least two 'false' on the stack,
                // this will just push a false again.
                self.push(!state);
                Some(!state)
            }
            None => None,
        }
    }

    /// Check if the stack is clean (i.e., all conditions are true).
    pub fn is_clean(&self) -> bool {
        self.stack.last().copied().unwrap_or(true)
    }
}

/// a C preprocessor implementation that preserves provenance information
/// when expanding macros and includes.
pub struct CPreprocessor {
    #[allow(dead_code)]
    path: PathBuf,
    snippets: Snippets,
    #[allow(dead_code)]
    defines: Definitions,
}

impl CPreprocessor {
    pub fn new(
        input: &SourceCode,
        path: &Path,
        user_defines: &HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let mut defines = Definitions::new();
        let datetime: DateTime<Local> = Local::now();
        // Format date as "Mmm dd yyyy", e.g. "Jan 19 2024". Date number is space-padded.
        let date = CppToken {
            text: format!("\"{}\"", datetime.format("%b %e %Y")),
            kind: CppTokenKind::String,
        };
        defines.insert(
            "__DATE__".to_string(),
            Definition::new(&[date], None, Provenance::SystemDefined),
        );
        // Format time as "hh:mm:ss", e.g. "13:45:30". Time numbers are zero-padded.
        let time = CppToken {
            text: format!("\"{}\"", datetime.format("%H:%M:%S")),
            kind: CppTokenKind::String,
        };
        defines.insert(
            "__TIME__".to_string(),
            Definition::new(&[time], None, Provenance::SystemDefined),
        );
        // Add user definitions from command line.
        // Must come after __DATE__ and __TIME__ to allow them to be overwritten.
        for (key, value) in user_defines {
            let replacement = CppTokenIterator::new(value)
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            defines.insert(
                key.clone(),
                Definition::new(&replacement, None, Provenance::UserDefined),
            );
        }
        // Preprocess the input.
        let mut snippets = Snippets::new();
        let mut if_stack = IfStack::new();
        for line in LogicalLines::from_source_code(input) {
            if let Some((_, directive)) = regex_captures!(r"\s*#\s*([a-z]+)", line.text()) {
                match directive {
                    "define" => {
                        if if_stack.is_clean() {
                            defines.handle_define(&line, path)?;
                        }
                    }
                    "undef" => {
                        if if_stack.is_clean() {
                            defines.handle_undef(&line)?;
                        }
                    }
                    "ifdef" => {
                        if_stack.push(defines.handle_ifdef(&line)?);
                    }
                    "ifndef" => {
                        if_stack.push(defines.handle_ifndef(&line)?);
                    }
                    "else" => {
                        // Expect possible whitespace, then 'else', possible whitespace, then newline
                        let mut iter = CppTokenIterator::new(line.text());
                        iter.consume_whitespace();
                        let token = iter
                            .consume_directive()
                            .context("Expected else directive")?;
                        if token.kind != CppTokenKind::Directive(CppDirectiveKind::Else) {
                            return Err(anyhow!("Expected else directive"));
                        }
                        iter.consume_whitespace();
                        let _ = iter
                            .consume_newline()
                            .context("Else directive should be on empty line")?;
                        if_stack
                            .toggle()
                            .ok_or_else(|| anyhow!("Encountered unexpected else directive"))?;
                    }
                    "endif" => {
                        // Expect possible whitespace, then 'endif', possible whitespace, then newline
                        let mut iter = CppTokenIterator::new(line.text());
                        iter.consume_whitespace();
                        let token = iter
                            .consume_directive()
                            .context("Expected endif directive")?;
                        if token.kind != CppTokenKind::Directive(CppDirectiveKind::Endif) {
                            return Err(anyhow!("Expected endif directive"));
                        }
                        iter.consume_whitespace();
                        let _ = iter
                            .consume_newline()
                            .context("Endif directive should be on empty line")?;
                        if_stack
                            .pop()
                            .ok_or_else(|| anyhow!("Encountered unexpected endif directive"))?;
                    }
                    _ => {
                        // Unknown directive, ignore line
                    }
                }
            } else {
                // Not a pre-processor directive line
                let mut iter = CppTokenIterator::new(line.text());
                while let Some(token) = iter.next() {
                    match token.kind {
                        CppTokenKind::Identifier => {
                            if !if_stack.is_clean() {
                                continue;
                            }
                            match defines.macro_kind(token.text) {
                                MacroKind::Function => {
                                    // Try to parse argument list
                                    if let Some(arglist) = iter.consume_arglist_invocation()? {
                                        let (definition, replacement) =
                                            defines.expand_function_macro(token.text, &arglist)?;
                                        snippets
                                            .push(&replacement, definition.provenance().clone());
                                        continue;
                                    } else {
                                        // No argument list, treat as normal identifier
                                    }
                                }
                                MacroKind::Object => {
                                    let (definition, replacement) =
                                        defines.expand_object_macro(token.text)?;
                                    snippets.push(&replacement, definition.provenance().clone());
                                    continue;
                                }
                                MacroKind::None => {
                                    // Not a macro, handle special cases and plain identifiers below
                                }
                            }
                            if token.text == "__LINE__" {
                                // Get the line number of the start of this token, accounting
                                // for line continuations.
                                let real_offset = line.offset(token.start);
                                let real_line = input.line_index(real_offset).to_string();
                                snippets.push(&real_line, Provenance::SystemDefined);
                            } else if token.text == "__FILE__" {
                                snippets.push(
                                    &format!("\"{}\"", path.to_string_lossy()),
                                    Provenance::SystemDefined,
                                );
                            } else {
                                let start = line.offset(token.start);
                                let end = line.offset(token.end);
                                snippets.push(token.text, Provenance::LocalText { start, end });
                            }
                        }
                        CppTokenKind::Directive(kind) => {
                            // Should not happen, directives are handled above.
                            return Err(anyhow!(
                                "Unexpected directive token {:?} in non-directive line",
                                kind
                            ));
                        }
                        CppTokenKind::Comment => {
                            // C-style comments are skipped in replacement text.
                        }
                        _ => {
                            if !if_stack.is_clean() {
                                continue;
                            }
                            let start = line.offset(token.start);
                            let end = line.offset(token.end);
                            snippets.push(token.text, Provenance::LocalText { start, end });
                        }
                    }
                }
            }
        }
        Ok(Self {
            path: path.to_path_buf(),
            snippets,
            defines,
        })
    }

    pub fn output(&self) -> String {
        self.snippets.collect()
    }
}

impl IntoIterator for CPreprocessor {
    type Item = Snippet;
    type IntoIter = std::vec::IntoIter<Snippet>;

    fn into_iter(self) -> Self::IntoIter {
        self.snippets.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;
    use lazy_regex::regex_is_match;
    use ruff_source_file::LineIndex;

    fn preprocess(code: &str) -> anyhow::Result<(String, Vec<Snippet>)> {
        let line_index = LineIndex::from_source_text(code);
        let code = SourceCode::new(code, &line_index);
        let mut user_defines = HashMap::new();
        user_defines.insert("__GNU__".to_string(), "".to_string());
        user_defines.insert("TEST".to_string(), 42.to_string());
        let preprocessor = CPreprocessor::new(&code, &PathBuf::from("test.f90"), &user_defines)?;
        Ok((preprocessor.output(), preprocessor.snippets.inner))
    }

    #[test]
    fn test_datetime() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            __DATE__
            __TIME__ UTC
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let output = output.replace('\n', " ");
        // Check that output matches the expected datetime format
        assert!(regex_is_match!(
            r#""[A-z]{3} [\s\d][\d] \d{4}" "\d{2}:\d{2}:\d{2}""#,
            &output
        ));
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 4);
        // __DATE__
        assert_eq!(snippets[0].provenance, Provenance::SystemDefined);
        // \n
        assert_eq!(
            snippets[1].provenance,
            Provenance::LocalText {
                start: TextSize::from(8),
                end: TextSize::from(9)
            }
        );
        // __TIME__
        assert_eq!(snippets[2].provenance, Provenance::SystemDefined);
        // " UTC"
        assert_eq!(
            snippets[3].provenance,
            Provenance::LocalText {
                start: TextSize::from(17),
                end: TextSize::from(21)
            }
        );
        Ok(())
    }

    #[test]
    fn test_object_macros() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define W 5
            #define X 10
            #define Y
            #define Z W,Y X
            #undef X
            program p
              integer :: X
              X = 12
              print *, Z, __FILE__, __LINE__, TEST
            end program p
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            program p
              integer :: X
              X = 12
              print *, 5, X, "test.f90", 9, 42
            end program p
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 9);
        // Z
        if let Provenance::FileDefined { start, end, path } = &snippets[1].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], "#define Z W,Y X\n");
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for Z");
        }
        // ", " following Z
        if let Provenance::LocalText { start, end } = &snippets[2].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following Z");
        }
        // __FILE__
        assert_eq!(snippets[3].provenance, Provenance::SystemDefined);
        // ", " following __FILE__
        if let Provenance::LocalText { start, end } = &snippets[4].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following __FILE__");
        }
        // __LINE__
        assert_eq!(snippets[5].provenance, Provenance::SystemDefined);
        // ", " following __LINE__
        if let Provenance::LocalText { start, end } = &snippets[6].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following __LINE__");
        }
        // TEST
        assert_eq!(snippets[7].provenance, Provenance::UserDefined);
        // Rest of code
        if let Provenance::LocalText { start, end } = &snippets[8].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], "\nend program p");
        } else {
            panic!("Expected LocalText provenance for rest of code");
        }
        Ok(())
    }

    #[test]
    fn test_function_macros() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define W 5
            #define foo( x ) (x + W)
            #define bar(x, y) x//y
            #define baz() 10
            program p
              implicit none
              integer, parameter :: foo = 1
              integer, parameter :: baz = 3
              print *, foo, foo(5), foo(foo + 2), foo(foo(7) + W)
              print *, bar("hello, ","world!")
              print *, baz, baz(), foo(baz())
            end program p
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            program p
              implicit none
              integer, parameter :: foo = 1
              integer, parameter :: baz = 3
              print *, foo, (5 + 5), (foo + 2 + 5), ((7 + 5) + 5 + 5)
              print *, "hello, "//"world!"
              print *, baz, 10, (10 + 5)
            end program p
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 13);
        // foo
        for i in [1, 3, 5, 11] {
            if let Provenance::FileDefined { start, end, path } = &snippets[i].provenance {
                assert_eq!(
                    &code[start.to_usize()..end.to_usize()],
                    "#define foo( x ) (x + W)\n"
                );
                assert_eq!(path, &PathBuf::from("test.f90"));
            } else {
                panic!("Expected FileDefined provenance for foo");
            }
        }
        if let Provenance::FileDefined { start, end, path } = &snippets[7].provenance {
            assert_eq!(
                &code[start.to_usize()..end.to_usize()],
                "#define bar(x, y) x//y\n"
            );
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for bar");
        }
        if let Provenance::FileDefined { start, end, path } = &snippets[9].provenance {
            assert_eq!(
                &code[start.to_usize()..end.to_usize()],
                "#define baz() 10\n"
            );
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for baz");
        }
        Ok(())
    }

    #[test]
    fn test_nested_function_macros() -> anyhow::Result<()> {
        // change foo(7) to foo(7) + W
        // Add foo(baz())
        let code = dedent!(
            r#"
            #define foo(x, y) (x + y)
            #define bar(x) foo(x, y)
            foo(bar(2), 10)
        "#
        );
        let (output, _) = preprocess(code)?;
        let expected = "((2 +  y) +  10)";
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_if() -> anyhow::Result<()> {
        // Note: definitions are expanded even in Fortran comments!
        let code = dedent!(
            r#"
            #ifdef X
            #  ifndef Y
            ! X !Y
            #    define Z 10
            #  else
            ! X Y
            #    define Z 20
            #  endif
            #else
            #  ifdef Y
            ! !X Y
            #    define Z 30
            #  else
            ! !X !Y
            #    define Z 40
            #  endif
            #endif
            program p
              print *, Z
            end program p
        "#
        );

        let (output, _) = preprocess(&["#define X", code].join("\n"))?;
        let expected = dedent!(
            r#"
            !  !Y
            program p
              print *, 10
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(&["#define X", "#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            !  
            program p
              print *, 20
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(&["#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            ! !X 
            program p
              print *, 30
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(code)?;
        let expected = dedent!(
            r#"
            ! !X !Y
            program p
              print *, 40
            end program p
        "#
        );
        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn test_unknown_directive() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define X test
            #unknown_directive
            end
        "#
        );
        let (output, _) = preprocess(code)?;
        assert_eq!(output.as_str(), "end");
        Ok(())
    }

    #[test]
    fn test_line_continuation() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #def\
            ine X \
            (1 + \
            2)
            print\
             *, X, __\
            LI\
            NE_\
            _
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            print *, (1 + 2), 6
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 4);
        // "print *, ""
        if let Provenance::LocalText { start, end } = &snippets[0].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], "print\\\n *, ");
        } else {
            panic!("Expected LocalText provenance for 'print *, '");
        }
        // X
        if let Provenance::FileDefined { start, end, path } = &snippets[1].provenance {
            assert_eq!(
                &code[start.to_usize()..end.to_usize()],
                "#def\\\nine X \\\n(1 + \\\n2)\n"
            );
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for X");
        }
        // ", "
        if let Provenance::LocalText { start, end } = &snippets[2].provenance {
            assert_eq!(&code[start.to_usize()..end.to_usize()], ", ");
        } else {
            panic!("Expected LocalText provenance for ', '");
        }
        // __LINE__
        assert_eq!(snippets[3].provenance, Provenance::SystemDefined);
        Ok(())
    }

    #[test]
    fn test_comments() -> anyhow::Result<()> {
        // WARNING: This test replicates behaviour that deviates from gfortran.
        //
        // gcc/gfortran (second line differs from this test):
        // x y z
        // xmerge(y,z)
        // xyz
        //
        // pcpp (all comments replaced by spaces):
        // x y z
        // x y z
        // x y z
        //
        // lfortran (agrees with this test):
        // x y z
        // xyz
        // xyz
        let code = dedent!(
            r#"
            #define merge(x, y) x/**/y
            merge(x, merge(y, z))
            merge(x,merge(y,z))
            x/* comment */y/* 
                            *another comment */z
            "#
        );
        let (output, _) = preprocess(code)?;
        let expected = dedent!(
            r#"
            x y z
            xyz
            xyz
        "#
        );
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_comments_and_escaped_newlines() -> anyhow::Result<()> {
        // Similar to test_comments, but with the addition of escaped newlines
        // lfortran begins to disagree, giving:
        // x   y z
        // x  yz
        // xy\
        // z
        //
        // gfortran and pcpp remain the same as in test_comments.
        let code = dedent!(
            r#"
            #define merge(x, y) \
              x/**/y
            merge(x, merge(y, \
            z))
            merge(x,merge(y,z))
            x/* comment */y/* 
                            *another comment */\
            z
            "#
        );
        let (output, _) = preprocess(code)?;
        let expected = dedent!(
            r#"
            x y z
            xyz
            xyz
        "#
        );
        assert_eq!(output, expected);
        Ok(())
    }
}
