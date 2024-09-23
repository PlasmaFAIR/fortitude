use crate::settings::Settings;
use crate::violation;
use crate::{BaseRule, TextRule, Violation};
use lazy_regex::regex_is_match;
/// Defines rules that govern line length.

pub struct LineTooLong {
    line_length: usize,
}

impl BaseRule for LineTooLong {
    fn new(settings: &Settings) -> Self {
        LineTooLong {
            line_length: settings.line_length,
        }
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

impl TextRule for LineTooLong {
    fn check(&self, source: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        for (idx, line) in source.split('\n').enumerate() {
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
                violations.push(violation!(&msg, idx + 1));
            }
        }
        violations
    }
}
