use crate::settings::Settings;
use crate::violation;
use crate::{Rule, TextRule, Violation};
/// Defines rules that enforce widely accepted whitespace rules.

pub struct TrailingWhitespace {}

impl Rule for TrailingWhitespace {
    fn new(_settings: &Settings) -> Self {
        TrailingWhitespace {}
    }

    fn explain(&self) -> &'static str {
        "
        Trailing whitespace is difficult to spot, and as some editors will remove it
        automatically while others leave it, it can cause unwanted 'diff noise' in
        shared projects.
        "
    }
}

impl TextRule for TrailingWhitespace {
    fn check(&self, source: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        for (idx, line) in source.split('\n').enumerate() {
            if line.ends_with(&[' ', '\t']) {
                violations.push(violation!("trailing whitespace", idx + 1));
            }
        }
        violations
    }
}
