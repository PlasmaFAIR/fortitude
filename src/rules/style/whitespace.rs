use ruff_source_file::SourceFile;

use crate::settings::Settings;
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
    fn check(&self, source: &SourceFile) -> Vec<Violation> {
        let mut violations = Vec::new();
        for (idx, line) in source.source_text().split('\n').enumerate() {
            if line.ends_with(&[' ', '\t']) {
                violations.push(Violation::from_start_end_line_col(
                    "trailing whitespace",
                    source,
                    idx,
                    line.trim_end().len(),
                    idx,
                    line.len(),
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
    use pretty_assertions::assert_eq;
    use ruff_source_file::SourceFileBuilder;

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
        let file = SourceFileBuilder::new("test", source).finish();

        let expected: Vec<Violation> = [(0, 13, 0, 15), (3, 23, 3, 24), (7, 3, 7, 7), (8, 0, 8, 3)]
            .iter()
            .map(|(start_line, start_col, end_line, end_col)| {
                Violation::from_start_end_line_col(
                    "trailing whitespace",
                    &file,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            })
            .collect();
        let rule = TrailingWhitespace::new(&default_settings());
        let actual = rule.check(&file);
        assert_eq!(actual, expected);
        Ok(())
    }
}
