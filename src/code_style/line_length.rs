use crate::core::{Method, Rule, Violation};
use crate::settings::Settings;
use crate::violation;
use regex::Regex;
/// Defines rules that govern line length.

pub struct EnforceMaxLineLength {}

fn enforce_max_line_length(source: &str, settings: &Settings) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Are we ending on a string or comment? If so, we'll allow it through, as
    // it may contain something like a long URL that cannot be reasonably split
    // across multiple lines.
    let re = Regex::new(r#"(["']\w*&?$)|(!.*$)|(^\w*&)"#).unwrap();

    for (idx, line) in source.split('\n').enumerate() {
        let len = line.len();
        if len > settings.line_length {
            if re.is_match(line) {
                continue;
            }
            let msg = format!(
                "line length of {}, exceeds maximum {}",
                len, settings.line_length
            );
            violations.push(violation!(&msg, idx + 1));
        }
    }
    violations
}

impl Rule for EnforceMaxLineLength {
    fn method(&self) -> Method {
        Method::Text(enforce_max_line_length)
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
