use lazy_regex::regex_replace_all;

fn blank_double_quote_string(s: &str) -> String {
    regex_replace_all!(r#"[^\s&"]*"#, s, |m: &str| " ".repeat(m.len())).into()
}

fn blank_single_quote_string(s: &str) -> String {
    regex_replace_all!(r#"[^\s&']*"#, s, |m: &str| " ".repeat(m.len())).into()
}

fn blank_comment(s: &str) -> String {
    "!".repeat(s.len())
}

/// Convert contents of strings to whitespace and comments to '!' so text rules won't match.
pub fn blank_comments_and_strings<S: AsRef<str>>(line: S) -> String {
    // Need to replace with the equivalent number of _bytes_.
    // (?ms) at the beginning sets the flags:
    // - 'm': multiline
    // - 's': dot matches newline
    regex_replace_all!(
        r#"(?m)("[^"]*"|'[^']*'|!.*$)"#,
        line.as_ref(),
        |_, m: &str| {
            match m.chars().next().unwrap() {
                '"' => blank_double_quote_string(m),
                '\'' => blank_single_quote_string(m),
                '!' => blank_comment(m),
                _ => unreachable!(),
            }
        }
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn blank_strings_and_comments() -> anyhow::Result<()> {
        // accented letters are two bytes long
        // the smiley is three bytes long
        let test = r#"
program p
  implicit none  ! super important
  write (*,*) "héllô &
    & 'wôrld'! 😀 &

    ! comments càn go here!
      & foo!", 'bàr', &
  & "baz", &
    ' lorum ǐpsum &
      & dolor sit     &
& àmet'
"#;
        let actual = blank_comments_and_strings(test);
        let expected = r#"
program p
  implicit none  !!!!!!!!!!!!!!!!!
  write (*,*) "        &
    &                &

                            
      &     ", '    ', &
  & "   ", &
    '              &
      &               &
&      '
"#;
        assert_eq!(expected, actual.as_str());

        Ok(())
    }
}
