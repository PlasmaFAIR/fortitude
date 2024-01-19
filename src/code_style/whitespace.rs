use crate::core::{Method, Rule, Violation};
use crate::violation;
use regex::Regex;
/// Defines rules that enforce widely accepted whitespace rules.

pub struct AvoidTrailingWhitespace {
    re: Regex,
}

impl AvoidTrailingWhitespace {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            re: Regex::new(r"[ \t]+$")?,
        })
    }

    fn rule(&self, number: usize, line: &str) -> Option<Violation> {
        if self.re.is_match(line) {
            Some(violation!("trailing whitespace", number))
        } else {
            None
        }
    }
}

impl Rule for AvoidTrailingWhitespace {
    fn method(&self) -> Method {
        Method::Line(Box::new(move |num, line| self.rule(num, line)))
    }

    fn explain(&self) -> &str {
        "
        Trailing whitespace is difficult to spot, and as some editors will remove it
        automatically while others leave it, it can cause unwanted 'diff noise' in
        shared projects.
        "
    }
}
