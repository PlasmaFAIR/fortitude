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
                violations.push(violation!(
                    "trailing whitespace",
                    idx + 1,
                    line.trim_end().len() + 1
                ));
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_trailing_whitespace() -> anyhow::Result<()> {
        // Careful of trailing whitespace in this string! That's the
        // point of the test! Also not using `dedent` here as it
        // messes with the whitespace-only line
        let source = "
program test  
  implicit none
  integer :: a(3) = [ & 
    1, &
    2, &
    3 &
  ]	
   
end program test
";
        let expected: Vec<Violation> = [(2, 13), (4, 24), (8, 4), (9, 1)]
            .iter()
            .map(|(line, col)| violation!("trailing whitespace", *line, *col))
            .collect();
        let rule = TrailingWhitespace::new(&default_settings());
        let actual = rule.check(source);
        assert_eq!(actual, expected);
        Ok(())
    }
}
