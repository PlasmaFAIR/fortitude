use std::{borrow::Cow, cell::OnceCell, fmt, ops::Deref};

use ruff_macros::CacheKey;
use ruff_source_file::{LineEnding, SourceFile, find_newline};
use ruff_text_size::TextSize;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{ast::FortitudeNode, traits::TextRanged};

#[derive(Debug, Clone)]
pub struct Stylist<'a> {
    source: Cow<'a, str>,
    capitalisation: Capitalisation,
    indentation: Indentation,
    quote: Quote,
    line_ending: OnceCell<LineEnding>,
}

impl<'a> Stylist<'a> {
    pub fn capitalisation(&self) -> Capitalisation {
        self.capitalisation
    }

    pub fn indentation(&self) -> &Indentation {
        &self.indentation
    }

    pub fn quote(&self) -> Quote {
        self.quote
    }

    pub fn line_ending(&self) -> LineEnding {
        *self.line_ending.get_or_init(|| {
            find_newline(&self.source)
                .map(|(_, ending)| ending)
                .unwrap_or_default()
        })
    }

    pub fn into_owned(self) -> Stylist<'static> {
        Stylist {
            source: Cow::Owned(self.source.into_owned()),
            capitalisation: self.capitalisation,
            indentation: self.indentation.clone(),
            quote: self.quote,
            line_ending: self.line_ending,
        }
    }

    pub fn from_ast(root: &Node, source: &'a SourceFile) -> Self {
        let first_statement = find_keyword(root);
        let capitalisation: Capitalisation = first_statement
            .map(|node| {
                node.to_text(source.source_text())
                    .unwrap_or_default()
                    .into()
            })
            .unwrap_or_default();
        let indentation = detect_indentation(&first_statement, source);
        let src = source.source_text();
        let quote = detect_quote(root, src);

        Self {
            source: Cow::Borrowed(src),
            capitalisation,
            indentation,
            quote,
            line_ending: OnceCell::default(),
        }
    }
}

fn detect_quote(root: &Node, src: &str) -> Quote {
    root.descendants()
        .find(|node| node.kind() == "string_literal")
        .map(|node| Quote::from_literal(&node, node.to_text(src).unwrap_or_default()))
        .unwrap_or_default()
}

/// Find the first "interesting" keyword
fn find_keyword<'a>(root: &'a Node) -> Option<Node<'a>> {
    root.named_descendants().find(|node| {
        matches!(
            node.kind(),
            "program" | "module" | "submodule" | "function" | "subroutine" | "interface"
        )
    })
}

/// Given a top-level entity, and then find the first statement that has
/// indentation longer than the indentation on that entity, and use the
/// difference
fn detect_indentation(first_statement: &Option<Node>, src: &SourceFile) -> Indentation {
    if first_statement.is_none() {
        return Indentation::default();
    }

    let first_statement = first_statement.unwrap();

    let current_indentation = first_statement.indentation(src);

    let indentation = first_statement
        .named_children(&mut first_statement.walk())
        .find_map(|node| {
            let new_indentation = node.indentation(src);
            if new_indentation.len() > current_indentation.len() {
                Some(new_indentation)
            } else {
                None
            }
        })
        .unwrap_or_default();

    Indentation::new(indentation.replacen(&current_indentation, "", 1))
}

/// The indentation style used in Fortran source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indentation(String);

impl Indentation {
    pub const fn new(indentation: String) -> Self {
        Self(indentation)
    }
}

impl Default for Indentation {
    fn default() -> Self {
        Indentation("    ".to_string())
    }
}

impl Indentation {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_char(&self) -> char {
        self.0.chars().next().unwrap()
    }
}

impl Deref for Indentation {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

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

impl fmt::Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, CacheKey, strum_macros::Display,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
#[derive(Default)]
pub enum Capitalisation {
    /// Use "lowercase" for keywords
    #[default]
    Lowercase,
    /// Use "UPPERCASE" for keywords
    Uppercase,
    /// Use "Titlecase" for keywords
    Titlecase,
}

impl From<&str> for Capitalisation {
    fn from(value: &str) -> Self {
        if value == value.to_uppercase() {
            Self::Uppercase
        } else if value.starts_with(|c: char| c.is_uppercase()) {
            Self::Titlecase
        } else {
            Self::Lowercase
        }
    }
}

pub trait ToCapitalisation {
    fn to_capitalisation(&self, capitalisation: Capitalisation) -> String;
}

impl ToCapitalisation for str {
    fn to_capitalisation(&self, capitalisation: Capitalisation) -> String {
        match capitalisation {
            Capitalisation::Lowercase => self.to_lowercase(),
            Capitalisation::Uppercase => self.to_uppercase(),
            Capitalisation::Titlecase => titlecase(self),
        }
    }
}

/// Converts a string to title case.
///
/// Every word (sequence of alphanumeric characters) has its first letter
/// uppercased. Non-alphanumeric characters act as word separators and are
/// preserved verbatim in the output.
pub fn titlecase(input: &str) -> String {
    let input = input.to_lowercase();
    let mut result = String::with_capacity(input.len());
    let mut start = 0;

    for (i, ch) in input.char_indices() {
        if !ch.is_alphanumeric() {
            if start < i {
                push_titlecase_word(&mut result, &input[start..i]);
            }
            result.push(ch);
            start = i + ch.len_utf8();
        }
    }

    if start < input.len() {
        push_titlecase_word(&mut result, &input[start..]);
    }

    result
}

/// Writes `word` into `buf` with its first character uppercased.
///
/// Writes directly into the caller's buffer — no intermediate allocation.
/// The word is assumed to already be in lowercase (caller's responsibility).
fn push_titlecase_word(buf: &mut String, word: &str) {
    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        // to_uppercase() yields an iterator because some chars expand
        // (e.g. 'ß' -> "SS"), so we can't just call .to_uppercase() as char.
        for ch in first.to_uppercase() {
            buf.push(ch);
        }
        // as_str() returns the remainder of the iterator as a &str slice —
        // zero copy, no reallocation.
        buf.push_str(chars.as_str());
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use ruff_source_file::{LineEnding, SourceFile, SourceFileBuilder, find_newline};
    use tree_sitter::{Parser, Tree};

    use crate::stylist::{Capitalisation, ToCapitalisation, titlecase};

    use super::{Indentation, Quote, Stylist};

    fn parse_snippet(code: &str) -> Result<(Tree, SourceFile)> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let file = SourceFileBuilder::new("test.f90", code).finish();
        Ok((tree, file))
    }

    #[test]
    fn indentation() {
        let contents = r"x = 1\nend";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation::default());

        let contents = r"
program foo
  implicit none
end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("  ".to_string()));

        let contents = r"
program foo
    implicit none
end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("    ".to_string()));

        let contents = r"
program foo
	implicit none
end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("\t".to_string()));

        let contents = r"
  program foo
    implicit none
  end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("  ".to_string()));

        let contents = r"
    program foo
        implicit none
    end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("    ".to_string()));

        let contents = r"
	program foo
		implicit none
	end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("\t".to_string()));

        let contents = r"
program foo
implicit none
  integer :: foo = 1
end
";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.indentation(), &Indentation("  ".to_string()));
    }

    #[test]
    fn quote() {
        let contents = r"x = 1\nend";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.quote(), Quote::default());

        let contents = r"x = '1'\nend";
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.quote(), Quote::Single);

        let contents = r#"x = "1"\nend"#;
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.quote(), Quote::Double);

        let contents = r#"s = "It's done."\nend"#;
        let parsed = parse_snippet(contents).unwrap();
        let stylist = Stylist::from_ast(&parsed.0.root_node(), &parsed.1);
        assert_eq!(stylist.quote(), Quote::Double);
    }

    #[test]
    fn line_ending() {
        let contents = "x = 1";
        assert_eq!(find_newline(contents).map(|(_, ending)| ending), None);

        let contents = "x = 1\n";
        assert_eq!(
            find_newline(contents).map(|(_, ending)| ending),
            Some(LineEnding::Lf)
        );

        let contents = "x = 1\r";
        assert_eq!(
            find_newline(contents).map(|(_, ending)| ending),
            Some(LineEnding::Cr)
        );

        let contents = "x = 1\r\n";
        assert_eq!(
            find_newline(contents).map(|(_, ending)| ending),
            Some(LineEnding::CrLf)
        );
    }

    #[test]
    fn titlecase_tests() {
        assert_eq!(titlecase(""), "");
        assert_eq!(titlecase("hello world"), "Hello World");
        assert_eq!(titlecase("RUST is GREAT"), "Rust Is Great");
        assert_eq!(titlecase("café au lait"), "Café Au Lait");
        assert_eq!(titlecase("hello--world"), "Hello--World");
        assert_eq!(titlecase("  leading"), "  Leading");
        assert_eq!(titlecase("île de france"), "Île De France");
        assert_eq!(titlecase("été indien"), "Été Indien");
        assert_eq!(titlecase("ßtraße"), "SStraße");
    }

    #[test]
    fn capitalisation() {
        assert_eq!(Capitalisation::from("PROGRAM"), Capitalisation::Uppercase);
        assert_eq!(Capitalisation::from("Program"), Capitalisation::Titlecase);
        assert_eq!(Capitalisation::from("program"), Capitalisation::Lowercase);
    }

    #[test]
    fn to_capitalisation() {
        assert_eq!(
            "program".to_capitalisation(Capitalisation::Uppercase),
            "PROGRAM"
        );
        assert_eq!(
            "program".to_capitalisation(Capitalisation::Titlecase),
            "Program"
        );
        assert_eq!(
            "PROGRAM".to_capitalisation(Capitalisation::Lowercase),
            "program"
        );

        assert_eq!(
            "end if".to_capitalisation(Capitalisation::Uppercase),
            "END IF"
        );
        assert_eq!(
            "end if".to_capitalisation(Capitalisation::Titlecase),
            "End If"
        );
        assert_eq!(
            "END IF".to_capitalisation(Capitalisation::Lowercase),
            "end if"
        );
    }
}
