use std::fmt;
use std::iter::Peekable;
use std::str::Utf8Error;

/// All valid 'punctuators' in Fortran.
/// These are consumed greedily by the tokenizer.
const PUNCTUATORS: &[&str] = &[
    "*", "+", "-", "/", "**", "=", // Arithmetic operators
    "==", "/=", "<", ">", "<=", ">=", // Comparison operators
    "(", "[", "(/", ")", "]", "/)", // Brackets
    ",", ".", "&", "%", "//", ";", ":", "::", // Others
];

/// The variant of each token.
#[derive(Debug, Clone, Copy, Eq, PartialEq, strum_macros::Display)]
pub enum CppDirectiveKind {
    Define,
    Undef,
    Include,
    If,
    Ifdef,
    Ifndef,
    Else,
    Elif,
    Endif,
    Pragma,
    Warning,
    Error,
    Stringification,
}

/// The variant of each token.
#[derive(Debug, Clone, Copy, Eq, PartialEq, strum_macros::Display)]
pub enum CppTokenKind {
    /// A variable or unexpanded macro token.
    Identifier,
    /// A number literal.
    Number,
    /// A string literal.
    String,
    /// A punctuation character.
    Punctuator,
    /// Whitespace, including spaces and tabs but excluding newlines.
    Whitespace,
    /// A newline, including LF, CR, and CRLF.
    Newline,
    /// A comment.
    Comment,
    /// An error occurred while tokenizing.
    Error,
    /// A preprocessor directive, including the leading `#`. Also captures
    /// stringification within macros.
    Directive(CppDirectiveKind),
    /// Token concatenation. Some compilers support `##` for this, but
    /// gfortran uses `/**/`.
    Concatenation, // TODO:
                   // Variadic
}

/// A token in a source file.
#[derive(Debug, Clone, Copy)]
pub struct CppToken<'a> {
    /// The text of the token.
    pub text: &'a str,
    /// The kind of token.
    pub kind: CppTokenKind,
    /// The beginning of the token in the source file.
    pub start: usize,
    /// The end of the token in the source file.
    pub end: usize,
}

impl fmt::Display for CppToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            text,
            kind,
            start,
            end,
        } = self;
        let text = text
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        write!(f, "{start} -> {end} | {kind} | `{text}`")
    }
}

/// An iterator over `&str` that returns tokens.
pub struct CppTokenIterator<'a> {
    /// Reference to the source string.
    source: &'a str,
    /// Internal iterator.
    iter: Peekable<std::str::Bytes<'a>>,
    /// Local byte offset counter
    offset: usize,
}

type CppTokenResult<'a> = Result<CppToken<'a>, Utf8Error>;

impl<'a> CppTokenIterator<'a> {
    /// The list of functions to call to consume tokens.  The order of the
    /// functions is important, as they are called in order.
    const FUNCS: [fn(&mut Self) -> Option<CppTokenResult<'a>>; 8] = [
        CppTokenIterator::consume_whitespace,
        CppTokenIterator::consume_newline,
        CppTokenIterator::consume_comment,
        CppTokenIterator::consume_string,
        CppTokenIterator::consume_directive,
        CppTokenIterator::consume_identifier,
        CppTokenIterator::consume_number,
        CppTokenIterator::consume_punctuator,
    ];

    /// Creates a new token iterator.
    pub fn new(source: &'a str) -> Self {
        let iter = source.bytes().peekable();
        let offset = 0;
        Self {
            source,
            iter,
            offset,
        }
    }

    fn step(&mut self) -> Option<u8> {
        let b = self.iter.next()?;
        self.offset += 1;
        Some(b)
    }

    /// Generate token from the given position to the current position.
    fn emit(&self, start: usize, kind: CppTokenKind) -> CppTokenResult<'a> {
        let end = self.offset;
        let text = &self.source[start..end];
        Ok(CppToken {
            text,
            kind,
            start,
            end,
        })
    }

    /// Skip the rest of the current line excluding the newline.
    pub fn skip_to_line_end(&mut self) {
        while !matches!(self.iter.peek(), Some(&b'\n') | Some(&b'\r') | None) {
            self.step();
        }
    }

    /// If the next token is a newline, consume it. Includes LF, CR, and CRLF.
    pub fn consume_newline(&mut self) -> Option<CppTokenResult<'a>> {
        let start = self.offset;
        match self.iter.peek() {
            Some(&b'\n') => {
                self.step();
                Some(self.emit(start, CppTokenKind::Newline))
            }
            Some(&b'\r') => {
                self.step();
                // Handle CRLF
                if self.iter.peek() == Some(&b'\n') {
                    self.step();
                }
                Some(self.emit(start, CppTokenKind::Newline))
            }
            _ => None,
        }
    }

    /// Consumes any amount of whitespace and combinations of tabs and spaces.
    /// Does not include newlines.
    pub fn consume_whitespace(&mut self) -> Option<CppTokenResult<'a>> {
        let start = self.offset;
        while matches!(self.iter.peek(), Some(&b' ') | Some(&b'\t')) {
            self.step();
        }
        if self.offset > start {
            Some(self.emit(start, CppTokenKind::Whitespace))
        } else {
            None
        }
    }

    /// Consume a comment until the end of the line.
    fn consume_comment(&mut self) -> Option<CppTokenResult<'a>> {
        if *self.iter.peek()? == b'!' {
            let start = self.offset;
            self.skip_to_line_end();
            Some(self.emit(start, CppTokenKind::Comment))
        } else {
            None
        }
    }

    /// If the next token is a string, consume it. Otherwise returns `None`.
    /// Handles both single and double quoted strings.
    /// Handles multiline strings with line continuations.
    /// Does not handle escaped quotes, instead treating them as
    /// string delimiters.
    fn consume_string(&mut self) -> Option<CppTokenResult<'a>> {
        let delimiter = match self.iter.peek() {
            Some(&b'\'') => b'\'',
            Some(&b'\"') => b'\"',
            _ => return None,
        };
        let start = self.offset;
        self.step();
        while self.iter.peek() != Some(&delimiter) {
            if self.iter.peek().is_none() {
                // Unterminated string
                return Some(self.emit(start, CppTokenKind::Error));
            }
            self.step();
        }
        self.step();
        Some(self.emit(start, CppTokenKind::String))
    }

    /// Consume a preprocessor directive, including the leading `#`.  Any amount
    /// of whitespace can be between the `#` and the directive name.  Also
    /// captures stringification within macros.
    pub fn consume_directive(&mut self) -> Option<CppTokenResult<'a>> {
        if self.iter.peek() != Some(&b'#') {
            return None;
        }
        let start = self.offset;
        self.step();
        // If the next character is another '#', it's a concatenation operator.
        if self.iter.peek() == Some(&b'#') {
            self.step();
            return Some(self.emit(start, CppTokenKind::Concatenation));
        }
        // Consume any whitespace after the '#'.
        let _ = self.consume_whitespace();
        // Consume an identifier for the directive name.
        match self.consume_identifier() {
            Some(Ok(directive)) => {
                let directive_kind = match directive.text {
                    "define" => CppDirectiveKind::Define,
                    "undef" => CppDirectiveKind::Undef,
                    "include" => CppDirectiveKind::Include,
                    "if" => CppDirectiveKind::If,
                    "ifdef" => CppDirectiveKind::Ifdef,
                    "ifndef" => CppDirectiveKind::Ifndef,
                    "elif" => CppDirectiveKind::Elif,
                    "else" => CppDirectiveKind::Else,
                    "endif" => CppDirectiveKind::Endif,
                    "pragma" => CppDirectiveKind::Pragma,
                    "warning" => CppDirectiveKind::Warning,
                    "error" => CppDirectiveKind::Error,
                    _ => CppDirectiveKind::Stringification,
                };
                Some(self.emit(start, CppTokenKind::Directive(directive_kind)))
            }
            _ => Some(self.emit(start, CppTokenKind::Error)),
        }
    }

    /// Consumes an identifier, such as a variable, function name, or macro
    /// name. These may include '$' characters to handle Fortran extensions.
    pub fn consume_identifier(&mut self) -> Option<CppTokenResult<'a>> {
        let first_char = self.iter.peek()?;
        if !first_char.is_ascii_alphabetic() && *first_char != b'_' && *first_char != b'$' {
            return None;
        }
        let start = self.offset;
        self.step();
        while let Some(&b) = self.iter.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'$' {
                self.step();
            } else {
                break;
            }
        }
        Some(self.emit(start, CppTokenKind::Identifier))
    }

    /// Consumes a 'preprocessing number', which is defined in the GCC docs as:
    ///
    /// A preprocessing number has a rather bizarre definition. The category
    /// includes all the normal integer and floating point constants one
    /// expects of C, but also a number of other things one might not
    /// initially recognize as a number. Formally, preprocessing numbers
    /// begin with an optional period, a required decimal digit, and then
    /// continue with any sequence of letters, digits, underscores, periods,
    /// and exponents. Exponents are the two-character sequences ‘e+’, ‘e-’,
    /// ‘E+’, ‘E-’, ‘p+’, ‘p-’, ‘P+’, and ‘P-’. (The exponents that begin
    /// with ‘p’ or ‘P’ are used for hexadecimal floating-point constants.)
    ///
    /// From experimentation, underscores are not actually allowed in gfortran.
    /// The exponents 'p' and 'P' are also not allowed, but 'd' and 'D' are.
    fn consume_number(&mut self) -> Option<CppTokenResult<'a>> {
        let first_char = *self.iter.peek()?;
        if first_char != b'.' && !first_char.is_ascii_digit() {
            return None;
        }
        let start = self.offset;
        self.step();
        // Handle optional leading period.
        // If it isn't followed by a digit, it's not a number.
        if first_char == b'.' && self.iter.peek().filter(|&x| x.is_ascii_digit()).is_none() {
            return Some(self.emit(start, CppTokenKind::Punctuator));
        }
        // Consume the rest of the number.
        while let Some(&b) = self.iter.peek() {
            match b {
                b'e' | b'E' | b'd' | b'D' => {
                    self.step();
                    if let Some(&next) = self.iter.peek() {
                        if next == b'+' || next == b'-' {
                            self.step();
                        }
                    }
                    continue;
                }
                _ => {
                    if b.is_ascii_digit() || b == b'.' {
                        self.step();
                        continue;
                    }
                    // End of number.
                    break;
                }
            }
        }
        Some(self.emit(start, CppTokenKind::Number))
    }

    /// Consumes a punctuator, such as '+', '==', or '(/'.
    fn consume_punctuator(&mut self) -> Option<CppTokenResult<'a>> {
        // From the starting position, find the longest string that
        // matches a punctuator.
        let start = self.offset;
        let mut end = start;
        while end < self.source.len()
            && PUNCTUATORS.iter().any(|op| {
                op.as_bytes()
                    .starts_with(&self.source.as_bytes()[start..end + 1])
            })
        {
            end += 1;
        }
        // Check that the string matches a punctuator.
        let text = &self.source.as_bytes()[start..end];
        if PUNCTUATORS.iter().any(|&op| op.as_bytes() == text) {
            let start = self.offset;
            // Advance the iterator to catch up to the copy and return the token.
            for _ in 0..text.len() {
                self.step();
            }
            Some(self.emit(start, CppTokenKind::Punctuator))
        } else {
            // Did not find a valid punctuator.
            None
        }
    }
}

impl<'a> Iterator for CppTokenIterator<'a> {
    type Item = CppTokenResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for f in Self::FUNCS.iter() {
            if let Some(token) = f(self) {
                return Some(token);
            }
        }
        if self.offset == self.source.len() {
            // End of file
            return None;
        }
        // Unhandled token found.
        let start = self.offset;
        for _ in &mut self.iter {
            // Consume the rest of the file.
        }
        Some(self.emit(start, CppTokenKind::Error))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;

    fn tokenize(input: &str) -> Vec<CppToken<'_>> {
        CppTokenIterator::new(input)
            .filter_map(|token| token.ok())
            .collect()
    }

    #[test]
    fn test_whitespace_and_newline_tokenization() {
        let input = " \t \n\t  \r \r\n  ";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 7);
        for (i, token) in tokens.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(token.kind, CppTokenKind::Whitespace);
            } else {
                assert_eq!(token.kind, CppTokenKind::Newline);
            }
        }
    }

    #[test]
    fn test_identifier_tokenization() {
        let input_vec = [
            "__IDENT__",
            "$dollar_ident",
            "_ident123",
            "ident_456",
            "ident$789",
        ];
        let input = input_vec.join(" ");
        let tokens = tokenize(input.as_str());
        assert_eq!(tokens.len(), 9);
        // Check that all identifier tokens are correctly identified.
        for (i, token) in tokens.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(token.kind, CppTokenKind::Identifier);
            } else {
                assert_eq!(token.kind, CppTokenKind::Whitespace);
            }
        }
        // Check that all tokens match the input.
        for (expected, token) in input_vec.iter().zip(tokens.iter().step_by(2)) {
            assert_eq!(token.text, *expected);
        }
    }

    #[test]
    fn test_string_tokenization() {
        let input = dedent!(
            r#"
            "string literal"
            'another string'
            "escaped "" quote"
            'another escaped '' quote'
            "continued &
                & string"
            'another continued &
                & string'
        "#
        );
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 13);
        let expected_kinds = [
            CppTokenKind::String,
            CppTokenKind::Newline,
            CppTokenKind::String,
            CppTokenKind::Newline,
            CppTokenKind::String,
            CppTokenKind::String,
            CppTokenKind::Newline,
            CppTokenKind::String,
            CppTokenKind::String,
            CppTokenKind::Newline,
            CppTokenKind::String,
            CppTokenKind::Newline,
            CppTokenKind::String,
            CppTokenKind::Newline,
        ];
        for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected_kind);
        }
    }

    #[test]
    fn test_comment_tokenization() {
        let input = dedent!(
            r#"
            __IDENT__ ! This is a comment
            !Another comment
            !Third comment
            __IDENT__!Fourth comment
        "#
        );
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 10);
        let expected_kinds = [
            CppTokenKind::Identifier,
            CppTokenKind::Whitespace,
            CppTokenKind::Comment,
            CppTokenKind::Newline,
            CppTokenKind::Comment,
            CppTokenKind::Newline,
            CppTokenKind::Comment,
            CppTokenKind::Newline,
            CppTokenKind::Identifier,
            CppTokenKind::Comment,
        ];
        for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected_kind);
        }
    }

    #[test]
    fn test_number_tokenization() {
        let input_vec = vec![
            "0", "123", "123.456", "1.23e10", "1.23E10", "1.23e+10", "1.23e-10", "1.23E+10",
            "1.23E-10", "1.23d10", "1.23D10", "1.23d+10", "1.23d-10", "1.23D+10", "1.23D-10",
            ".23", ".23e10", ".23E10", ".23e+10", ".23e-10", ".23d10", ".23D10", ".23D+10",
            ".23D-10", "12.", "12.e10", "12.E10", "12.e+10", "12.e-10", "12.d10", "12.D10",
            "12.D+10", "12.D-10",
        ];
        let number_of_tokens = input_vec.len();
        let input = input_vec.join(" ");
        // Test integers
        let tokens = tokenize(input.as_str());
        assert_eq!(tokens.len(), number_of_tokens * 2 - 1); // Each number + whitespac
                                                            // Check that all number tokens are correctly identified.
        for (i, token) in tokens.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(token.kind, CppTokenKind::Number);
            } else {
                assert_eq!(token.kind, CppTokenKind::Whitespace);
            }
        }
        // Check that all tokens match the input.
        for (expected, token) in input_vec.iter().zip(tokens.iter().step_by(2)) {
            assert_eq!(token.text, *expected);
        }
    }

    #[test]
    fn test_number_vs_period_tokenization() {
        // Test a single period, which should be a punctuator.
        let input = ".5 5. .";
        let tokens = tokenize(input);
        let expected_kinds = [
            CppTokenKind::Number,
            CppTokenKind::Whitespace,
            CppTokenKind::Number,
            CppTokenKind::Whitespace,
            CppTokenKind::Punctuator,
        ];
        assert_eq!(tokens.len(), expected_kinds.len());
        for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected_kind);
        }
    }

    #[test]
    fn test_number_with_kind_tokenization() {
        let input = "1_8 1.0e10_8";
        let tokens = tokenize(input);
        let expected_kinds = [
            CppTokenKind::Number,
            CppTokenKind::Identifier,
            CppTokenKind::Whitespace,
            CppTokenKind::Number,
            CppTokenKind::Identifier,
        ];
        for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected_kind);
            if expected_kind == &CppTokenKind::Identifier {
                assert_eq!(token.text, "_8");
            }
        }
    }

    #[test]
    fn test_directive_tokenization() {
        let input = dedent!(
            r#"
            #define MAX 100
            #  undef MIN
            #include "file.h"
            #   include <file.h>
            #if defined(MAX)
            #else
            #endif
        "#
        );
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 32);
        let expected_kinds = [
            CppTokenKind::Directive(CppDirectiveKind::Define),
            CppTokenKind::Whitespace,
            CppTokenKind::Identifier, // MAX
            CppTokenKind::Whitespace,
            CppTokenKind::Number, // 100
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::Undef),
            CppTokenKind::Whitespace,
            CppTokenKind::Identifier, // MIN
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::Include),
            CppTokenKind::Whitespace,
            CppTokenKind::String, // "file.h"
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::Include),
            CppTokenKind::Whitespace,
            CppTokenKind::Punctuator, // <
            CppTokenKind::Identifier, // file
            CppTokenKind::Punctuator, // .
            CppTokenKind::Identifier, // h
            CppTokenKind::Punctuator, // >
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::If),
            CppTokenKind::Whitespace,
            CppTokenKind::Identifier, // defined
            CppTokenKind::Punctuator, // (
            CppTokenKind::Identifier, // MAX
            CppTokenKind::Punctuator, // )
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::Else),
            CppTokenKind::Newline,
            CppTokenKind::Directive(CppDirectiveKind::Endif),
        ];
        for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(token.kind, *expected_kind);
        }
    }
}
