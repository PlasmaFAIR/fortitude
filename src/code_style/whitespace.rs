use crate::core::{Method, Rule, Violation};
use crate::violation;
use regex::Regex;
/// Defines rule that enforces widely accepted whitespace rules.

fn avoid_trailing_whitespace(number: usize, line: &str) -> Option<Violation> {
    let re = Regex::new(r"[ \t]+$").unwrap();
    if re.is_match(line) {
        Some(violation!("trailing whitespace", number))
    } else {
        None
    }
}

pub struct AvoidTrailingWhitespace {}

impl Rule for AvoidTrailingWhitespace {
    fn method(&self) -> Method {
        Method::Line(avoid_trailing_whitespace)
    }

    fn explain(&self) -> &str {
        "Trailing whitespace is difficult to spot, and when working on shared projects
        it can cause unwanted 'diff noise'."
    }
}
