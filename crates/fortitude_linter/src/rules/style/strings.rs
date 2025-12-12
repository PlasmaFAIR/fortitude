use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextLen, TextSize};
use settings::Quote;
use tree_sitter::Node;

use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};

/// ## What does it do?
/// Catches use of single- or double-quoted strings, depending on the value of
/// [`check.strings.quotes`] option.
///
/// ## Why is this bad?
/// For consistency, all strings should be either single-quoted or double-quoted.
/// Exceptions are made for strings containing escaped quotes.
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
    fn check(
        settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let preferred_quote = settings.strings.quotes;
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
            return some_vec!(
                Diagnostic::from_node(
                    Self {
                        preferred_quote,
                        contains_escaped_quotes: false,
                    },
                    node,
                )
                .with_fix(Fix::safe_edits(edit_start, [edit_end]))
            );
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
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
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

        // Because the kind appears as a child node, the interesting
        // literal bit doesn't start at the beginning of the node
        let (start, kind) = if let Some(kind) = node.child_by_field_name("kind") {
            let kind_text = kind.to_text(src.source_text())?;
            (kind_text.len() + 1, format!("{kind_text}_"))
        } else {
            (0, "".to_string())
        };

        let end = text.text_len() - TextSize::new(1);
        let contents = &text[start + 1..end.to_usize()];
        let fixed = format!(
            "{kind}{quote}{value}{quote}",
            quote = quote_style.opposite().as_char(),
            value = unescape_string(contents, quote_style.as_char())
        );

        let edit = node.edit_replacement(src, fixed);
        // Offset start of node by kind, if any
        let range = node
            .textrange()
            .add_start(TextSize::try_from(start).unwrap());
        some_vec!(Diagnostic::new(Self, range).with_fix(Fix::safe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["string_literal"]
    }
}

fn unescape_string(haystack: &str, quote: char) -> String {
    let mut fixed_contents = String::with_capacity(haystack.len());

    let mut chars = haystack.chars().peekable();
    let mut seen_quote = false;
    while let Some(char_) = chars.next() {
        // Not a quote, or the previous character was a quote, which
        // we've removed
        if char_ != quote || seen_quote {
            fixed_contents.push(char_);
            seen_quote = false;
            continue;
        }
        // If we're at the end of the line
        let Some(next_char) = chars.peek() else {
            fixed_contents.push(char_);
            continue;
        };
        // Remove first of two consecutive quotes
        if *next_char == quote {
            seen_quote = true;
            continue;
        }
        fixed_contents.push(char_);
    }
    fixed_contents
}

pub mod settings {
    use super::*;
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use serde::{Deserialize, Serialize};
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, CacheKey)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    #[derive(Default)]
    pub enum Quote {
        /// Use double quotes.
        #[default]
        Double,
        /// Use single quotes.
        Single,
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
