use anyhow::anyhow;
use ruff_text_size::TextSize;
use std::borrow::{Borrow, ToOwned};
use std::fmt;
use std::iter::Peekable;
use std::str::FromStr;

/// All valid 'punctuators'. Includes all ASCII punctuation except for single
/// and double quotes.  Multichar operators like '==' and '/=' are handled here
/// as sequences of single-character punctuators. As the preprocessor doesn't
/// understand Fortran comments, the exclamation mark '!' is included as a valid
/// punctuator.
const PUNCTUATORS: &[u8] = b"*+-/*=<>([)]{},.&|%;:!?~^|\\`@#$";

/// The variant of each token.
#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "lowercase")]
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
}

/// The variant of each token.
#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display)]
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
    /// A preprocessor block comment, starting with `/*` and ending with `*/`.
    Comment,
    /// A preprocessor directive, including the leading `#`.
    Directive(CppDirectiveKind),
    /// Something that isn't recognised.
    Other,
}

/// A token in a source file. References the source string.
#[derive(Debug)]
pub struct CppTokenRef<'a> {
    /// The text of the token.
    pub text: &'a str,
    /// The kind of token.
    pub kind: CppTokenKind,
    /// The beginning of the token in the source file.
    pub start: TextSize,
    /// The end of the token in the source file.
    pub end: TextSize,
}

impl fmt::Display for CppTokenRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            text,
            kind,
            start,
            end,
        } = self;
        let start = start.to_u32();
        let end = end.to_u32();
        let text = text
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        write!(f, "{start} -> {end} | {kind} | `{text}`")
    }
}

/// A token from a source file. Owns its text.
#[derive(Debug, Clone)]
pub struct CppToken {
    /// The text of the token.
    pub text: String,
    /// The kind of token.
    pub kind: CppTokenKind,
}

impl<'a> ToOwned for CppTokenRef<'a> {
    type Owned = CppToken;

    fn to_owned(&self) -> Self::Owned {
        CppToken {
            text: self.text.to_string(),
            kind: self.kind.clone(),
        }
    }
}

impl<'a> Borrow<CppTokenRef<'a>> for CppToken {
    fn borrow(&self) -> &CppTokenRef<'a> {
        // This cannot be implemented safely, as it would require
        // creating a reference to an object that doesn't exist.
        unimplemented!("Cannot borrow CppToken as CppTokenRef");
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

impl<'a> CppTokenIterator<'a> {
    /// The list of functions to call to consume tokens. The order of the
    /// functions is important, as they are called in order. Directives
    /// should be handled separately, and via this method will be
    /// interpreted as '#' punctuators followed by identifiers.
    const FUNCS: [fn(&mut Self) -> Option<CppTokenRef<'a>>; 7] = [
        CppTokenIterator::consume_whitespace,
        CppTokenIterator::consume_newline,
        CppTokenIterator::consume_identifier,
        CppTokenIterator::consume_number,
        CppTokenIterator::consume_string,
        CppTokenIterator::consume_comment,
        CppTokenIterator::consume_punctuator_or_other,
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
    fn emit(&self, start: usize, kind: CppTokenKind) -> CppTokenRef<'a> {
        let end = self.offset;
        let text = &self.source[start..end];
        CppTokenRef {
            text,
            kind,
            start: TextSize::from(start as u32),
            end: TextSize::from(end as u32),
        }
    }

    /// If the next token is a newline, consume it. Includes LF, CR, and CRLF.
    pub fn consume_newline(&mut self) -> Option<CppTokenRef<'a>> {
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
    pub fn consume_whitespace(&mut self) -> Option<CppTokenRef<'a>> {
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
    fn consume_comment(&mut self) -> Option<CppTokenRef<'a>> {
        // C-style comments start with '/*' and end with '*/'. They can span
        // multiple lines in the source file, but when converting to logical
        // lines they will be treated as a single line.
        if self.iter.peek() == Some(&b'/') {
            let mut clone_iter = self.iter.clone();
            clone_iter.next(); // Consume '/'
            if clone_iter.peek() == Some(&b'*') {
                // It's a C comment.
                let start = self.offset;
                self.step(); // Consume '/'
                self.step(); // Consume '*'
                while let Some(&b) = self.iter.peek() {
                    if b == b'*' {
                        self.step();
                        if self.iter.peek() == Some(&b'/') {
                            self.step(); // Consume '/'
                            return Some(self.emit(start, CppTokenKind::Comment));
                        }
                    } else {
                        self.step();
                    }
                }
                // Unterminated comment at end of file. Return comment anyway.
                return Some(self.emit(start, CppTokenKind::Comment));
            }
        }
        None
    }

    /// If the next token is a string, consume it. Otherwise returns `None`.
    /// Handles both single and double quoted strings. Does not handle escaped
    /// quotes, instead treating them as string delimiters. Does not handle
    /// multiline strings, instead treating newlines as terminating characters.
    /// This is consistent with gfortran's preprocessor behavior.
    fn consume_string(&mut self) -> Option<CppTokenRef<'a>> {
        let delimiter = match self.iter.peek() {
            Some(&b'\'') => b'\'',
            Some(&b'\"') => b'\"',
            _ => return None,
        };
        let start = self.offset;
        self.step();
        while self.iter.peek() != Some(&delimiter) {
            let peek = self.iter.peek();
            if peek.is_none() || peek == Some(&b'\n') || peek == Some(&b'\r') {
                // Unterminated string or end of line.
                return Some(self.emit(start, CppTokenKind::String));
            }
            self.step();
        }
        self.step();
        Some(self.emit(start, CppTokenKind::String))
    }

    /// Consumes an identifier, such as a variable, function name, or macro
    /// name. These may not include '$' characters, although some compilers
    /// allow them.
    pub fn consume_identifier(&mut self) -> Option<CppTokenRef<'a>> {
        let first_char = self.iter.peek()?;
        if !first_char.is_ascii_alphabetic() && *first_char != b'_' {
            return None;
        }
        let start = self.offset;
        self.step();
        while let Some(&b) = self.iter.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
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
    fn consume_number(&mut self) -> Option<CppTokenRef<'a>> {
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

    /// Consumes a punctuator.
    fn consume_punctuator_or_other(&mut self) -> Option<CppTokenRef<'a>> {
        let start = self.offset;
        let next_byte = self.step()?;
        if PUNCTUATORS.contains(&next_byte) {
            Some(self.emit(start, CppTokenKind::Punctuator))
        } else {
            // Ensure we always consume something, but make sure it's on
            // a utf-8 boundary.
            let mut word = vec![next_byte];
            loop {
                if std::str::from_utf8(&word).is_ok() {
                    break;
                }
                match self.step() {
                    Some(b) => word.push(b),
                    None => break,
                }
            }
            Some(self.emit(start, CppTokenKind::Other))
        }
    }

    /// Consume a preprocessor directive, including the leading `#`. Any amount
    /// of whitespace can be between the `#` and the directive name. Should not
    /// be called alongside other token consumption methods, as directives must
    /// be at the start of a logical line.
    pub fn consume_directive(&mut self) -> anyhow::Result<CppTokenRef<'a>> {
        let start = self.offset;
        // Consume any whitespace at the start of the line.
        self.consume_whitespace();
        // Consume '#'. Where this is called, the '#' must be the next character,
        // so we can step safely.
        self.step();
        // Consume any whitespace after the '#'.
        self.consume_whitespace();
        // Consume an identifier for the directive name.
        match self.consume_identifier() {
            Some(directive) => {
                if let Ok(directive_kind) = CppDirectiveKind::from_str(directive.text) {
                    Ok(self.emit(start, CppTokenKind::Directive(directive_kind)))
                } else {
                    Err(anyhow!("Unknown directive: {}", directive.text))
                }
            }
            _ => Err(anyhow!("Expected identifier after '#'")),
        }
    }

    /// Consumes a parenthesized, comma-separated list of identifiers.
    /// Used for function-like macro definitions.
    /// Returns `Ok(None)` if the next token is not an opening parenthesis.
    /// Returns `Err` if the argument list is malformed.
    pub fn consume_arglist_definition(&mut self) -> anyhow::Result<Option<Vec<String>>> {
        let mut args = Vec::new();
        // Expect an opening parenthesis.
        if self.iter.peek() != Some(&b'(') {
            return Ok(None);
        }
        self.step(); // Consume '('
        if self.iter.peek() == Some(&b')') {
            self.step(); // Consume ')'
            return Ok(Some(args)); // Empty argument list
        }
        loop {
            // Optional whitespace.
            let _ = self.consume_whitespace();
            // Expect an identifier.
            let ident = match self.next() {
                Some(token) => {
                    if token.kind != CppTokenKind::Identifier {
                        return Err(anyhow!(
                            "Expected identifier in argument list, found {}",
                            token.text
                        ));
                    }
                    token.text.to_string()
                }
                None => return Err(anyhow!("Unexpected end of input in argument list")),
            };
            args.push(ident);
            // Optional whitespace.
            let _ = self.consume_whitespace();
            match self.iter.peek() {
                Some(&b',') => {
                    self.step(); // Consume ','
                }
                Some(&b')') => {
                    self.step(); // Consume ')'
                    break;
                }
                Some(x) => return Err(anyhow!("Invalid character in argument list: {}", x)),
                None => return Err(anyhow!("Unexpected end of input in argument list")),
            }
        }
        Ok(Some(args))
    }

    /// Consumes a parenthesized, comma-separated list-of-lists of tokens.
    /// Each argument can consist of multiple tokens.
    /// Used for function-like macro calls.
    /// Returns `Ok(None)` if the next token is not an opening parenthesis. It is valid
    /// to use a function macro identifier without an argument list, but it is treated
    /// as a normal identifier in that case.
    /// Returns `Err` if the argument list is malformed.
    pub fn consume_arglist_invocation(&mut self) -> anyhow::Result<Option<Vec<Vec<CppToken>>>> {
        let mut args = vec![Vec::new()];
        // Expect an opening parenthesis.
        if self.iter.peek() != Some(&b'(') {
            return Ok(None);
        }
        self.step(); // Consume '('
        // Empty argument list
        if self.iter.peek() == Some(&b')') {
            self.step(); // Consume ')'
            return Ok(Some(args));
        }
        let mut bracket_nesting = 1;
        loop {
            match self.next() {
                Some(token) => {
                    if token.kind == CppTokenKind::Punctuator {
                        match token.text {
                            "," if bracket_nesting == 1 => {
                                // Start a new argument.
                                args.push(Vec::new());
                                continue;
                            }
                            "(" => {
                                bracket_nesting += 1;
                            }
                            ")" => {
                                bracket_nesting -= 1;
                                if bracket_nesting == 0 {
                                    // End of argument list.
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    // Add token to the current argument.
                    if let Some(current_arg) = args.last_mut() {
                        current_arg.push(token.to_owned());
                    } else {
                        return Err(anyhow!("Internal error: no current argument"));
                    }
                }
                None => return Err(anyhow!("Unexpected end of input in argument list")),
            }
        }
        Ok(Some(args))
    }
}

impl<'a> Iterator for CppTokenIterator<'a> {
    type Item = CppTokenRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for f in Self::FUNCS.iter() {
            if let Some(token) = f(self) {
                return Some(token);
            }
        }
        // If the function consume_punctuator_or_other returned None,
        // we must be at the end of the file.
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;

    fn tokenize(input: &str) -> Vec<CppTokenRef<'_>> {
        CppTokenIterator::new(input).collect()
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
        // Identifiers can include letters, digits, and underscores,
        // but must start with a letter or underscore. Some compilers
        // allow dollar signs in identifiers, but they are treated as
        // separate punctuators here.
        let input = "__IDENT__ _ident123 ident_456 $dollar_ident 23y";
        let tokens = tokenize(input);
        let expected = [
            (CppTokenKind::Identifier, "__IDENT__"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "_ident123"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "ident_456"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Punctuator, "$"),
            (CppTokenKind::Identifier, "dollar_ident"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Number, "23"),
            (CppTokenKind::Identifier, "y"),
        ];
        assert_eq!(tokens.len(), expected.len());
        for (token, expected) in tokens.iter().zip(expected.iter()) {
            assert_eq!(token.kind, expected.0);
            assert_eq!(token.text, expected.1);
        }
    }

    #[test]
    fn test_string_tokenization() {
        // The preprocessor does not handle escaped quotes or multiline strings.
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
        let expected = [
            (CppTokenKind::String, "\"string literal\""),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::String, "'another string'"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::String, "\"escaped \""),
            (CppTokenKind::String, "\" quote\""),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::String, "'another escaped '"),
            (CppTokenKind::String, "' quote'"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::String, "\"continued &"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::Whitespace, "    "),
            (CppTokenKind::Punctuator, "&"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "string"),
            (CppTokenKind::String, "\""),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::String, "'another continued &"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::Whitespace, "    "),
            (CppTokenKind::Punctuator, "&"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "string"),
            (CppTokenKind::String, "'"),
        ];
        assert_eq!(tokens.len(), expected.len());
        for (token, expected) in tokens.iter().zip(expected.iter()) {
            assert_eq!(token.kind, expected.0);
            assert_eq!(token.text, expected.1);
        }
    }

    #[test]
    fn test_comment_tokenization() {
        // Fortran comments are not properly tokenized, but C-style comments are.
        let input = dedent!(
            r#"
            __IDENT__ ! comment
            __IDENT__!comment
            X/**/Y
            X/* hello world! */Y
        "#
        );
        let tokens = tokenize(input);
        let expected = [
            (CppTokenKind::Identifier, "__IDENT__"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Punctuator, "!"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "comment"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::Identifier, "__IDENT__"),
            (CppTokenKind::Punctuator, "!"),
            (CppTokenKind::Identifier, "comment"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::Identifier, "X"),
            (CppTokenKind::Comment, "/**/"),
            (CppTokenKind::Identifier, "Y"),
            (CppTokenKind::Newline, "\n"),
            (CppTokenKind::Identifier, "X"),
            (CppTokenKind::Comment, "/* hello world! */"),
            (CppTokenKind::Identifier, "Y"),
        ];
        assert_eq!(tokens.len(), expected.len());
        for (token, expected) in tokens.iter().zip(expected.iter()) {
            assert_eq!(token.kind, expected.0);
            assert_eq!(token.text, expected.1);
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
    fn test_multibyte_utf8_tokenization() {
        // Fortran only allows non-ASCII characters in comments and strings.
        // If the preprocessor understood Fortran, this would mean we wouldn't
        // need to handle them directly, but as it doesn't, it's possible for
        // the preprocessor to encounter them.
        let input = "Combien de chats avez-vous acheté ?";
        let tokens = tokenize(input);
        let expected = [
            (CppTokenKind::Identifier, "Combien"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "de"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "chats"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "avez"),
            (CppTokenKind::Punctuator, "-"),
            (CppTokenKind::Identifier, "vous"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Identifier, "achet"),
            (CppTokenKind::Other, "é"),
            (CppTokenKind::Whitespace, " "),
            (CppTokenKind::Punctuator, "?"),
        ];
        assert_eq!(tokens.len(), expected.len());
        for (token, expected) in tokens.iter().zip(expected.iter()) {
            assert_eq!(token.kind, expected.0);
            assert_eq!(token.text, expected.1);
        }
    }
}
