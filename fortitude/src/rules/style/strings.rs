use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextLen, TextSize};
use settings::Quote;
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};

/// ## What does it do?
/// Catches use of single- or double-quoted strings, depending on the value of
/// [`check.strings.quotes`] option.
///
/// ## Why is this bad?
/// For consistency, all strings should be either single-quoted or double-quoted.
/// Exceptions are made for strings containing quotes.
///
/// Fixes are not currently available for strings containing escaped quotes
/// (`"''"` or `""""`).
///
/// ## Example
/// ```f90
/// foo = 'bar'
/// ```
///
/// Assuming `quotes` is set to `double`, use instead:
/// ```f90
/// foo = "bar"
/// ```
///
/// ## Options
/// - `check.strings.quotes`
#[derive(ViolationMetadata)]
pub(crate) struct BadQuoteString {
    preferred_quote: Quote,
    contains_escaped_quotes: bool,
}

impl Violation for BadQuoteString {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        match self.preferred_quote {
            Quote::Double => "String uses single quotes but double quotes preferred".to_string(),
            Quote::Single => "String uses double quotes but single quotes preferred".to_string(),
        }
    }

    fn fix_title(&self) -> Option<String> {
        if self.contains_escaped_quotes {
            return None;
        }
        let title = match self.preferred_quote {
            Quote::Double => "Replace single quotes with double quotes",
            Quote::Single => "Replace double quotes with single quotes",
        };
        Some(title.to_string())
    }
}

impl AstRule for BadQuoteString {
    fn check(settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let preferred_quote = settings.check.strings.quotes;
        let bad_quote = preferred_quote.opposite();

        let text = node.to_text(src.source_text())?;
        if text.starts_with(bad_quote.as_char())
            && text.ends_with(bad_quote.as_char())
            && !text.contains(preferred_quote.as_char())
        {
            // Search for occurrence of escaped single quotes within the string.
            // These are double single quotes, e.g. "''"
            if text.contains(bad_quote.escaped()) && text.len() > 2 {
                return Some(vec![Diagnostic::from_node(
                    Self {
                        preferred_quote,
                        contains_escaped_quotes: true,
                    },
                    node,
                )]);
            }

            let start_byte = node.start_textsize();
            let end_byte = node.end_textsize();
            let edit_start = Edit::replacement(
                preferred_quote.as_char().to_string(),
                start_byte,
                start_byte + TextSize::from(1),
            );
            let edit_end = Edit::replacement(
                preferred_quote.as_char().to_string(),
                end_byte - TextSize::from(1),
                end_byte,
            );
            return some_vec!(Diagnostic::from_node(
                Self {
                    preferred_quote,
                    contains_escaped_quotes: false,
                },
                node,
            )
            .with_fix(Fix::safe_edits(edit_start, [edit_end])));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["string_literal"]
    }
}

/// ## What it does
/// Checks for strings that include escaped quotes that can be removed if the
/// quote style is changed.
///
/// ## Why is this bad?
/// It's preferable to avoid escaped quotes in strings. By changing the
/// outer quote style, you can avoid escaping inner quotes.
///
/// ## Example
/// ```f90
/// foo = 'bar''s'
/// ```
///
/// Use instead:
/// ```f90
/// foo = "bar's"
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct AvoidableEscapedQuote;

impl AlwaysFixableViolation for AvoidableEscapedQuote {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Avoidable escaped quotes".to_string()
    }

    fn fix_title(&self) -> String {
        "Change outer quotes to avoid escaping inner quotes".to_string()
    }
}

impl AstRule for AvoidableEscapedQuote {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let text = node.to_text(src.source_text())?;
        if text.len() <= 2 {
            return None;
        }
        let quote_style = Quote::from_literal(node, text);

        if !text.contains(quote_style.escaped()) || text.contains(quote_style.opposite().as_char())
        {
            return None;
        }

        let end = text.text_len() - TextSize::new(1);
        let contents = &text[1..end.to_usize()];
        let fixed = format!(
            "{quote}{value}{quote}",
            quote = quote_style.opposite().as_char(),
            value = unescape_string(contents, quote_style.as_char())
        );

        let edit = node.edit_replacement(src, fixed);
        some_vec!(Diagnostic::from_node(Self, node).with_fix(Fix::safe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["string_literal"]
    }
}

fn unescape_string(haystack: &str, quote: char) -> String {
    let mut fixed_contents = String::with_capacity(haystack.len());

    let mut chars = haystack.chars().peekable();
    while let Some(char_) = chars.next() {
        if char_ != quote {
            fixed_contents.push(char_);
            continue;
        }
        // If we're at the end of the line
        let Some(next_char) = chars.peek() else {
            fixed_contents.push(char_);
            continue;
        };
        // Remove quote escape
        if *next_char == quote {
            continue;
        }
        fixed_contents.push(char_);
    }
    fixed_contents
}

pub(crate) mod settings {
    use super::*;
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, CacheKey)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub enum Quote {
        /// Use double quotes.
        Double,
        /// Use single quotes.
        Single,
    }

    impl Default for Quote {
        fn default() -> Self {
            Self::Double
        }
    }

    impl TryFrom<char> for Quote {
        type Error = &'static str;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '"' => Ok(Self::Double),
                '\'' => Ok(Self::Single),
                _ => Err("not a quote"),
            }
        }
    }

    impl Quote {
        #[must_use]
        pub const fn opposite(self) -> Self {
            match self {
                Self::Double => Self::Single,
                Self::Single => Self::Double,
            }
        }

        #[must_use]
        pub const fn escaped(self) -> &'static str {
            match self {
                Self::Double => r#""""#,
                Self::Single => r#"''"#,
            }
        }

        /// Get the character used to represent this quote.
        pub const fn as_char(self) -> char {
            match self {
                Self::Double => '"',
                Self::Single => '\'',
            }
        }

        pub fn from_literal(node: &Node, text: &str) -> Self {
            let mut start = TextSize::new(0);
            if let Some(kind) = node.child_by_field_name("kind") {
                start = kind.textrange().len() + TextSize::new(1);
            }
            let first_quote = text
                .chars()
                .nth(start.to_usize())
                .expect("couldn't slice string literal correctly");
            Quote::try_from(first_quote).expect("string literal doesn't begin with a quote")
        }
    }

    impl Display for Quote {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Double => write!(f, "double"),
                Self::Single => write!(f, "single"),
            }
        }
    }

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub quotes: Quote,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.strings",
                fields = [self.quotes]
            }
            Ok(())
        }
    }
}
