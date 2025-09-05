use crate::cpp_tokens::{CppDirectiveKind, CppTokenIterator, CppTokenKind};
use anyhow::Context;
use chrono::prelude::*;
use std::collections::HashMap;

pub struct CPreprocessor<'a> {
    pub input: &'a str,
    pub output: String,
    defines: HashMap<String, String>,
    iter: CppTokenIterator<'a>,
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
            defines,
            iter,
        }
    }

    pub fn preprocess(&mut self) -> anyhow::Result<()> {
        while let Some(token) = self.iter.next() {
            let token = token?;
            match token.kind {
                CppTokenKind::Identifier => {
                    if let Some(replacement) = self.defines.get(token.text) {
                        self.output.push_str(replacement);
                        continue;
                    }
                    if token.text == "__LINE__" {
                        self.output.push_str(&self.iter.line().to_string());
                    } else {
                        self.output.push_str(token.text);
                    }
                }
                CppTokenKind::Directive(kind) => {
                    self.handle_directive(kind)?;
                }
                _ => {
                    self.output.push_str(token.text);
                }
            }
        }
        Ok(())
    }

    fn handle_directive(&mut self, kind: CppDirectiveKind) -> anyhow::Result<()> {
        match kind {
            CppDirectiveKind::Define => self.handle_define(),
            CppDirectiveKind::Undef => self.handle_undef(),
            _ => {
                // Unhandled directive, skip the line.
                self.iter.skip_line();
                Ok(())
            }
        }
    }

    fn handle_define(&mut self) -> anyhow::Result<()> {
        // Expect whitespace, then an identifier
        let _ = self
            .iter
            .consume_whitespace()
            .context("Expected whitespace")?;
        let key = self
            .iter
            .consume_identifier()
            .context("Unexpected identifier")??;
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
        // Expect whitespace, then an identifier
        let _ = self
            .iter
            .consume_whitespace()
            .context("Expected whitespace")?;
        let key = self
            .iter
            .consume_identifier()
            .context("Unexpected identifier")??;
        self.defines
            .remove(key.text)
            .context(format!("Cannot undef undefined identifier: {}", key.text))?;
        Ok(())
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
            #undef X
            #define Z W, X Y
            program p
              integer :: X
              X = 12
              print *, Z, __FILE__, __LINE__
            end program p

        "#
        );
        insta::assert_snapshot!(preprocess(code)?);
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
