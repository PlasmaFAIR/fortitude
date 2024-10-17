use crate::settings::Settings;
use crate::{Rule, TextRule, Violation};
use lazy_regex::regex_is_match;
use ruff_source_file::SourceFile;
/// Defines rules that govern line length.

pub struct LineTooLong {
    line_length: usize,
}

impl Rule for LineTooLong {
    fn new(settings: &Settings) -> Self {
        LineTooLong {
            line_length: settings.line_length,
        }
    }

    fn explain(&self) -> &'static str {
        "
        Long lines are more difficult to read, and may not fit on some developers'
        terminals. The line continuation character '&' may be used to split a long line
        across multiple lines, and overly long expressions may be broken down into
        multiple parts.

        The maximum line length can be changed using the flag `--line-length=N`. The
        default maximum line length is 100 characters. This is a fair bit more than the
        traditional 80, but due to the verbosity of modern Fortran it can sometimes be
        difficult to squeeze lines into that width, especially when using large indents
        and multiple levels of indentation.

        Note that the Fortran standard states a maximum line length of 132 characters,
        and while some modern compilers will support longer lines, for portability it
        is recommended to stay beneath this limit.
        "
    }
}

impl TextRule for LineTooLong {
    fn check(&self, source: &SourceFile) -> Vec<Violation> {
        let mut violations = Vec::new();
        for (idx, line) in source.source_text().split('\n').enumerate() {
            let len = line.len();
            if len > self.line_length {
                // Are we ending on a string or comment? If so, we'll allow it through, as it may
                // contain something like a long URL that cannot be reasonably split across multiple
                // lines.
                if regex_is_match!(r#"(["']\w*&?$)|(!.*$)|(^\w*&)"#, line) {
                    continue;
                }
                let msg = format!(
                    "line length of {}, exceeds maximum {}",
                    len, self.line_length
                );
                violations.push(Violation::from_start_end_line_col(
                    msg,
                    source,
                    idx,
                    self.line_length,
                    idx,
                    len,
                ))
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use crate::test_file;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_line_too_long() -> anyhow::Result<()> {
        let source = test_file(
            "
            program test
              use some_really_long_module_name, only : integer_working_precision
              implicit none
              integer(integer_working_precision), parameter, dimension(1) :: a = [1]
            end program test
            ",
        );

        let line_length = 20;
        let short_line_settings = Settings { line_length };
        let expected: Vec<Violation> = [(2, line_length, 2, 68, 68), (4, line_length, 4, 72, 72)]
            .iter()
            .map(|(start_line, start_col, end_line, end_col, length)| {
                Violation::from_start_end_line_col(
                    format!("line length of {length}, exceeds maximum {line_length}"),
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            })
            .collect();
        let rule = LineTooLong::new(&short_line_settings);
        let actual = rule.check(&source);
        assert_eq!(actual, expected);
        Ok(())
    }
}
