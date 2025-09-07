use crate::cpp_tokens::{CppDirectiveKind, CppTokenIterator, CppTokenKind};
use anyhow::{anyhow, Context};
use chrono::prelude::*;
use std::collections::HashMap;

pub struct CPreprocessor<'a> {
    pub input: &'a str,
    pub output: String,
    iter: CppTokenIterator<'a>,
    defines: HashMap<String, String>,
    if_stack: Vec<bool>,
    // TODO: mapping of byte offsets between input and output
}

impl<'a> CPreprocessor<'a> {
    pub fn new(input: &'a str, filename: &str) -> Self {
        let output = String::new();
        let mut defines = HashMap::new();
        let datetime: DateTime<Local> = Local::now();
        defines.insert("__FILE__".to_string(), format!("\"{}\"", filename));
        // Format date as "Mmm dd yyyy", e.g. "Jan 19 2024". Date number is space-padded.
        defines.insert(
            "__DATE__".to_string(),
            format!("\"{}\"", datetime.format("%b %e %Y")),
        );
        // Format time as "hh:mm:ss", e.g. "13:45:30". Time numbers are zero-padded.
        defines.insert(
            "__TIME__".to_string(),
            format!("\"{}\"", datetime.format("%H:%M:%S")),
        );
        let iter = CppTokenIterator::new(input);
        Self {
            input,
            output,
            iter,
            defines,
            if_stack: Vec::new(),
        }
    }

    fn push(&mut self, s: &str) {
        if *self.if_stack.last().unwrap_or(&true) {
            self.output.push_str(s);
        }
    }

    pub fn preprocess(&mut self) -> anyhow::Result<()> {
        while let Some(token) = self.iter.next() {
            let token = token?;
            match token.kind {
                CppTokenKind::Identifier => {
                    if let Some(replacement) = self.defines.get(token.text) {
                        // Copying contents of self.push here to get around borrow checker
                        if *self.if_stack.last().unwrap_or(&true) {
                            self.output.push_str(replacement);
                        }
                        continue;
                    }
                    if token.text == "__LINE__" {
                        self.push(&self.iter.line().to_string());
                    } else {
                        self.push(token.text);
                    }
                }
                CppTokenKind::Directive(kind) => {
                    self.handle_directive(kind)?;
                }
                _ => {
                    self.push(token.text);
                }
            }
        }
        Ok(())
    }

    fn handle_directive(&mut self, kind: CppDirectiveKind) -> anyhow::Result<()> {
        match kind {
            CppDirectiveKind::Define => self.handle_define(),
            CppDirectiveKind::Undef => self.handle_undef(),
            CppDirectiveKind::Ifdef => self.handle_ifdef(false),
            CppDirectiveKind::Ifndef => self.handle_ifdef(true),
            CppDirectiveKind::Else => self.handle_else(),
            CppDirectiveKind::Endif => self.handle_endif(),
            _ => {
                // Unhandled directive, skip the line.
                self.iter.skip_line();
                Ok(())
            }
        }
    }

    fn handle_define(&mut self) -> anyhow::Result<()> {
        // If the if_stack end in false, ignore and continue
        if self.if_stack.last() == Some(&false) {
            self.iter.skip_line();
            return Ok(());
        }
        // Expect whitespace, then an identifier
        let _ = self
            .iter
            .consume_whitespace()
            .context("Expected whitespace")?;
        let key = self
            .iter
            .consume_identifier()
            .context("Expected identifier")??;
        // Optional whitespace
        let _ = self.iter.consume_whitespace();
        let mut value = String::new();
        for token in self.iter.by_ref() {
            let token = token?;
            match token.kind {
                CppTokenKind::Identifier => {
                    if let Some(replacement) = self.defines.get(token.text) {
                        value.push_str(replacement);
                    } else {
                        value.push_str(token.text);
                    }
                }
                CppTokenKind::Newline => {
                    break;
                }
                _ => value.push_str(token.text),
            }
        }
        // TODO: handle redefines properly
        self.defines.insert(key.text.to_string(), value);
        Ok(())
    }

    fn handle_undef(&mut self) -> anyhow::Result<()> {
        // If the if_stack end in false, ignore and continue
        if self.if_stack.last() == Some(&false) {
            self.iter.skip_line();
            return Ok(());
        }
        // Expect whitespace, then an identifier
        let _ = self
            .iter
            .consume_whitespace()
            .context("Expected whitespace")?;
        let key = self
            .iter
            .consume_identifier()
            .context("Expected identifier")??;
        // Expect nothing else on line
        self.iter.consume_whitespace();
        let _ = self
            .iter
            .consume_newline()
            .context("Malformed undef directive")?;

        self.defines
            .remove(key.text)
            .context(format!("Cannot undef undefined identifier: {}", key.text))?;
        Ok(())
    }

    fn handle_ifdef(&mut self, ifndef: bool) -> anyhow::Result<()> {
        // If already in a false branch, don't bother checking and
        // set false instead.
        if self.if_stack.last() == Some(&false) {
            self.if_stack.push(false);
            self.iter.skip_line();
            return Ok(());
        }
        // Expect whitespace, then an identifier
        let _ = self
            .iter
            .consume_whitespace()
            .context("Expected whitespace")?;
        let key = self
            .iter
            .consume_identifier()
            .context("Expected identifier")??;
        // Expect possible whitespace, then newline
        self.iter.consume_whitespace();
        let n = if ifndef { "n" } else { "" };
        let _ = self
            .iter
            .consume_newline()
            .context(format!("Malformed if{}def", n))?;
        // Determine whether the following block should be included
        let mut include_next_block = self.defines.contains_key(key.text);
        if ifndef {
            include_next_block = !include_next_block;
        }
        self.if_stack.push(include_next_block);
        Ok(())
    }

    fn handle_else(&mut self) -> anyhow::Result<()> {
        // Expect nothing else on the line
        self.iter.consume_whitespace();
        let _ = self
            .iter
            .consume_newline()
            .context("Else directive should be on empty line")?;
        // If there are at least two 'false' on the stack, do nothing and continue.
        let false_depth = self.if_stack.iter().rev().take_while(|&x| !x).count();
        println!("{false_depth}");
        if false_depth > 1 {
            return Ok(());
        }
        // Toggle last on the stack
        match self.if_stack.pop() {
            Some(state) => {
                self.if_stack.push(!state);
                Ok(())
            }
            None => {
                // if_stack was empty, raise error
                Err(anyhow!("Encountered unexpected else directive"))
            }
        }
    }

    fn handle_endif(&mut self) -> anyhow::Result<()> {
        // Expect nothing else on the line
        self.iter.consume_whitespace();
        let _ = self
            .iter
            .consume_newline()
            .context("Endif directive should be on empty line")?;

        match self.if_stack.pop() {
            Some(_) => Ok(()),
            None => Err(anyhow!("Encountered unexpected endif directive")),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;
    use lazy_regex::regex_is_match;

    fn preprocess(code: &str) -> anyhow::Result<String> {
        let mut preprocessor = CPreprocessor::new(code, "test.f90");
        preprocessor.preprocess()?;
        Ok(preprocessor.output)
    }

    #[test]
    fn test_datetime() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            __DATE__
            __TIME__ UTC
        "#
        );
        let output = preprocess(code)?.replace('\n', " ");
        // Check that output matches the expected datetime format
        assert!(regex_is_match!(
            r#""[A-z]{3} [\s\d][\d] \d{4}" "\d{2}:\d{2}:\d{2}""#,
            &output
        ));
        Ok(())
    }

    #[test]
    fn test_defines() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define W 5
            #define X 10
            #define Y
            #define Z W, X Y
            #undef X
            program p
              integer :: X
              X = 12
              print *, Z, __FILE__, __LINE__
            end program p

        "#
        );
        let output = preprocess(code)?;
        let expected = dedent!(
            r#"
            program p
              integer :: X
              X = 12
              print *, 5, 10 , "test.f90", 9
            end program p
        "#
        );
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_if() -> anyhow::Result<()> {
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

        let output = preprocess(&["#define X", code].join("\n"))?;
        let expected = dedent!(
            r#"
            ! X !Y
            program p
              print *, 10
            end program p
        "#
        );
        assert_eq!(output, expected);

        let output = preprocess(&["#define X", "#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            ! X Y
            program p
              print *, 20
            end program p
        "#
        );
        assert_eq!(output, expected);

        let output = preprocess(&["#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            ! !X Y
            program p
              print *, 30
            end program p
        "#
        );
        assert_eq!(output, expected);

        let output = preprocess(code)?;
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
            #define X
            #unknown_directive
            end
        "#
        );
        let output = preprocess(code)?;
        assert_eq!(output.as_str(), "end");
        Ok(())
    }
}
