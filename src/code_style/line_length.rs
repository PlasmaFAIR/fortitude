use crate::core::{Method, Rule, Violation};
use crate::settings::Settings;
use crate::violation;
use regex::Regex;
/// Defines rules that govern line length.

pub struct EnforceMaxLineLength {
    max_len: usize,
    re: Regex,
}

impl EnforceMaxLineLength {
    pub fn new(settings: &Settings) -> anyhow::Result<Self> {
        // Regex matches possible exceptions to the rule. Strings and comments might
        // exceed maximum line length, which may be due to them containing URLs that
        // cannot be shrunk down to fit.
        // - End in an string, possibly with a line continuation
        // - End in a comment
        // - Start line with a string continuation
        // Regex may need refining, this might let some bad lines through.
        let max_len = settings.line_length;
        if settings.strict && max_len > 132 {
            anyhow::bail!("Under strict mode, the maximum possible line length is 132")
        } else {
            Ok(Self {
                max_len: settings.line_length,
                re: Regex::new(r#"(["']\w*&?$)|(!.*$)|(^\w*&)"#)?,
            })
        }
    }

    fn rule(&self, number: usize, line: &str) -> Option<Violation> {
        let len = line.len();
        if len > self.max_len {
            // Are we ending on a string or comment? If so, allow it through, as it may
            // contain something like a long URL that cannot be reasonably split across
            // multiple lines.
            if self.re.is_match(line) {
                None
            } else {
                let msg = format!("line length of {}, exceeds maximum {}", len, self.max_len);
                Some(violation!(&msg, number))
            }
        } else {
            None
        }
    }
}

impl Rule for EnforceMaxLineLength {
    fn method(&self) -> Method {
        Method::Line(Box::new(move |num, line| self.rule(num, line)))
    }

    fn explain(&self) -> &str {
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
