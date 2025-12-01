use crate::cpp_tokens::{CppDirectiveKind, CppToken, CppTokenIterator, CppTokenKind};
use anyhow::{Context, anyhow};
use chrono::prelude::*;
use lazy_regex::regex_captures;
use ruff_source_file::{OneIndexed, SourceCode};
use ruff_text_size::TextSize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Enum describing where a snippet came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// A system-defined macro, e.g. __DATE__ or __TIME__.
    SystemDefined,
    /// A user-defined macro, e.g. passed in via command line.
    UserDefined,
    /// A macro defined in the source file.
    FileDefined {
        start: usize,
        end: usize,
        path: PathBuf,
    },
    /// Plain text from an included file.
    IncludeText { path: PathBuf },
    /// Plain text in the source file, not from a macro.
    LocalText { start: usize, end: usize },
}

/// A snippet of text with provenance information.
pub struct Snippet {
    text: String,
    provenance: Provenance,
}

impl Snippet {
    pub fn extend(&mut self, other: &Snippet) -> anyhow::Result<()> {
        match (&self.provenance, &other.provenance) {
            (
                Provenance::LocalText {
                    start: self_start,
                    end: self_end,
                },
                Provenance::LocalText {
                    start: other_start,
                    end: other_end,
                },
            ) => {
                if self_end == other_start {
                    self.text.push_str(&other.text);
                    self.provenance = Provenance::LocalText {
                        start: *self_start,
                        end: *other_end,
                    };
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Can only extend snippets with contiguous provenance"
                    ))
                }
            }
            _ => Err(anyhow!(
                "Can only extend snippets with LocalText provenance"
            )),
        }
    }
}

/// A collection of snippets.
pub struct Snippets {
    // Collected snippets
    inner: Vec<Snippet>,
}

impl Default for Snippets {
    fn default() -> Self {
        Self::new()
    }
}

impl Snippets {
    pub fn new() -> Self {
        Snippets { inner: Vec::new() }
    }

    pub fn push(&mut self, text: &str, provenance: Provenance) {
        let snippet = Snippet {
            text: text.to_string(),
            provenance,
        };
        if let Some(last) = self.inner.last_mut() {
            if last.extend(&snippet).is_ok() {
                return;
            }
        }
        self.inner.push(snippet);
    }

    pub fn collect(&self) -> String {
        self.inner
            .iter()
            .map(|s| s.text.as_str())
            .collect::<String>()
    }
}

/// A logical line of code, which may span multiple physical lines due to
/// line continuations. Tracks the byte offset of each location of the
/// logical line.
pub struct LogicalLine<'a> {
    /// The text of the logical line.
    text: Cow<'a, str>,
    /// The byte offsets of each character in the logical line
    /// relative to the start of the source file.
    /// TODO: Optimise this, only store start and any discontinuities
    byte_offsets: Vec<usize>,
    /// The number of real lines spanned by this logical line.
    span: usize,
}

impl<'a> LogicalLine<'a> {
    pub fn new(src: &'a SourceCode, start_line: usize) -> LogicalLine<'a> {
        let line_index = OneIndexed::from_zero_indexed(start_line);
        let line_text = src.line_text(line_index);
        let line_count = src.line_count();
        let line_offset: usize = src.line_start(line_index).into();
        let trimmed = line_text.trim_end();
        if let Some(continued) = trimmed.strip_suffix('\\') {
            let mut logical_line = LogicalLine {
                text: Cow::Borrowed(continued),
                byte_offsets: (line_offset..line_offset + continued.len()).collect(),
                span: 1,
            };
            if start_line + 1 < line_count {
                logical_line.extend(src, start_line + 1); // recursive
            }
            logical_line
        } else {
            LogicalLine {
                text: Cow::Borrowed(line_text),
                byte_offsets: (line_offset..line_offset + line_text.len()).collect(),
                span: 1,
            }
        }
    }

    fn extend(&mut self, src: &'a SourceCode, line: usize) -> &Self {
        self.span += 1;
        let line_index = OneIndexed::from_zero_indexed(line);
        let line_text = src.line_text(line_index);
        let line_offset: usize = src.line_start(line_index).into();
        let trimmed = line_text.trim_end();
        if let Some(continued) = trimmed.strip_suffix('\\') {
            self.text.to_mut().push_str(continued);
            let end_offset = line_offset + continued.len();
            self.byte_offsets.extend(line_offset..end_offset);
            if line + 1 < src.line_count() {
                self.extend(src, line + 1)
            } else {
                self
            }
        } else {
            self.text.to_mut().push_str(line_text);
            let end_offset = line_offset + line_text.len();
            self.byte_offsets.extend(line_offset..end_offset);
            self
        }
    }

    pub fn offset(&self, index: usize) -> Option<usize> {
        // handle the case where index is at the end of the text
        if index == self.text.len() {
            return self.byte_offsets.last().copied().map(|v| v + 1);
        }
        self.byte_offsets.get(index).copied()
    }

    pub fn offset_range(&self) -> Option<(usize, usize)> {
        if self.byte_offsets.is_empty() {
            None
        } else {
            Some((
                *self.byte_offsets.first().unwrap(),
                self.byte_offsets.last().unwrap() + 1,
            ))
        }
    }

    pub fn span(&self) -> usize {
        self.span
    }
}

pub struct LogicalLines<'a> {
    lines: Vec<LogicalLine<'a>>,
}

impl<'a> LogicalLines<'a> {
    pub fn new(src: &'a SourceCode) -> LogicalLines<'a> {
        let mut lines = Vec::new();
        let line_count = src.line_count();
        let mut line_index = 0;
        while line_index < line_count {
            let line = LogicalLine::new(src, line_index);
            line_index += line.span();
            lines.push(line);
        }
        LogicalLines { lines }
    }
}

impl<'a> IntoIterator for LogicalLines<'a> {
    type Item = LogicalLine<'a>;
    type IntoIter = std::vec::IntoIter<LogicalLine<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

/// A stack to track conditional compilation state.
pub struct IfStack {
    stack: Vec<bool>,
}

impl Default for IfStack {
    fn default() -> Self {
        Self::new()
    }
}

impl IfStack {
    /// Create a new, empty IfStack.
    pub fn new() -> Self {
        IfStack { stack: Vec::new() }
    }

    /// Push a new conditional state onto the stack. If
    /// the stack is already false, push false.
    pub fn push(&mut self, state: bool) {
        if self.is_clean() {
            self.stack.push(state);
        } else {
            self.stack.push(false);
        }
    }

    /// Pop the top conditional state from the stack.
    pub fn pop(&mut self) -> Option<bool> {
        self.stack.pop()
    }

    /// Toggle the top conditional state on the stack.
    /// If the stack has more than one false, does nothing.
    pub fn toggle(&mut self) -> Option<bool> {
        match self.pop() {
            Some(state) => {
                // If there are at least two 'false' on the stack,
                // this will just push a false again.
                self.push(!state);
                Some(!state)
            }
            None => None,
        }
    }

    /// Check if the stack is clean (i.e., all conditions are true).
    pub fn is_clean(&self) -> bool {
        self.stack.last().copied().unwrap_or(true)
    }
}

/// An object macro definition.
/// TODO, replacement should be a list of tokens, not a string.
/// TODO, implement function-like macros.
pub struct Definition {
    replacement: Vec<CppToken>,
    args: Option<Vec<String>>,
    provenance: Provenance,
}

/// Enum used to identify macro types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MacroKind {
    Object,
    Function,
    None,
}

/// A mapping of macro names to their definitions.
pub struct Definitions {
    inner: HashMap<String, Definition>,
}

impl Default for Definitions {
    fn default() -> Self {
        Self::new()
    }
}

impl Definitions {
    pub fn new() -> Self {
        Definitions {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Definition> {
        self.inner.get(key)
    }

    fn expand_macro_tokens(&self, tokens: &[CppToken]) -> anyhow::Result<String> {
        let mut result = String::new();
        let mut iter = tokens.iter().peekable();
        while let Some(token) = iter.next() {
            if token.kind == CppTokenKind::Identifier {
                match self.macro_kind(&token.text) {
                    MacroKind::Function => {
                        if let Some(next) = iter.peek() {
                            if next.kind == CppTokenKind::Punctuator && next.text == "(" {
                                // Found argument list
                                let mut arglist = Vec::new();
                                iter.next(); // Consume '('
                                // Check for empty argument list
                                if let Some(next) = iter.peek() {
                                    if next.kind == CppTokenKind::Punctuator && next.text == ")" {
                                        iter.next(); // Consume ')'
                                        // Empty argument list
                                        let (_, replacement) =
                                            self.expand_function_macro(&token.text, &arglist)?;
                                        result.push_str(&replacement);
                                        continue;
                                    }
                                }
                                arglist.push(Vec::new());
                                let mut bracket_nesting = 1;
                                for token in iter.by_ref() {
                                    if token.kind == CppTokenKind::Punctuator {
                                        match token.text.as_str() {
                                            "," if bracket_nesting == 1 => {
                                                arglist.push(Vec::new());
                                                continue;
                                            }
                                            "(" => {
                                                bracket_nesting += 1;
                                            }
                                            ")" => {
                                                bracket_nesting -= 1;
                                                if bracket_nesting == 0 {
                                                    break;
                                                }
                                            }
                                            _ => {
                                                // fallthrough to push token below
                                            }
                                        }
                                    }
                                    arglist.last_mut().unwrap().push(token.clone());
                                }
                                let (_, replacement) =
                                    self.expand_function_macro(&token.text, &arglist)?;
                                result.push_str(&replacement);
                                continue;
                            }
                        } else {
                            // No argument list, treat as normal identifier below
                        }
                    }
                    MacroKind::Object => {
                        let (_, replacement) = self.expand_object_macro(&token.text)?;
                        result.push_str(&replacement);
                        continue;
                    }
                    MacroKind::None => {
                        // Not a macro, handle as plain token below
                    }
                }
            }
            if token.kind == CppTokenKind::Comment {
                continue;
            }
            result.push_str(&token.text);
        }
        Ok(result)
    }

    pub fn expand_object_macro(&self, key: &str) -> anyhow::Result<(&Definition, String)> {
        let definition = self.inner.get(key).context("Internal: Macro not defined")?;
        let result = self.expand_macro_tokens(&definition.replacement)?;
        Ok((definition, result))
    }

    pub fn expand_function_macro(
        &self,
        key: &str,
        args: &[Vec<CppToken>],
    ) -> anyhow::Result<(&Definition, String)> {
        let definition = self
            .inner
            .get(key)
            .context("Internal: Function macro not found")?;
        let def_args = definition
            .args
            .as_ref()
            .context("Internal: Expected function macro argument list")?;
        if def_args.len() != args.len() {
            return Err(anyhow!(
                "Function macro argument count mismatch, {key}, {def_args:?}, {args:?}"
            ));
        }
        // Perform substitutions on first pass
        let mut substituted = Vec::new();
        for token in &definition.replacement {
            if token.kind == CppTokenKind::Identifier {
                if let Some(pos) = def_args.iter().position(|arg| arg == &token.text) {
                    // Replace with corresponding argument
                    substituted.extend(args[pos].iter().cloned());
                    continue;
                }
            }
            substituted.push(token.clone());
        }
        // Expand as usual
        let result = self.expand_macro_tokens(&substituted)?;
        Ok((definition, result))
    }

    pub fn insert(&mut self, key: String, definition: Definition) {
        self.inner.insert(key, definition);
    }

    pub fn remove(&mut self, key: &str) -> Option<Definition> {
        self.inner.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn macro_kind(&self, key: &str) -> MacroKind {
        if let Some(definition) = self.inner.get(key) {
            if definition.args.is_some() {
                MacroKind::Function
            } else {
                MacroKind::Object
            }
        } else {
            MacroKind::None
        }
    }

    pub fn handle_define(&mut self, line: &LogicalLine, path: &Path) -> anyhow::Result<()> {
        // Expect possible whitespace, then 'define'.
        let mut iter = CppTokenIterator::new(&line.text);
        iter.consume_whitespace();
        let directive = iter
            .consume_directive()
            .context("Expected define directive")?;
        if directive.kind != CppTokenKind::Directive(CppDirectiveKind::Define) {
            return Err(anyhow!("Expected define directive"));
        }
        // Expect whitespace, then an identifier
        let _ = iter.consume_whitespace().context("Expected whitespace")?;
        let key = iter.consume_identifier().context("Expected identifier")?;
        // Get optional argument list
        let args = iter.consume_arglist_definition()?;
        // Optional whitespace
        iter.consume_whitespace();
        // Get token list of replacement text
        let mut replacement = Vec::new();
        for token in iter.by_ref() {
            if token.kind == CppTokenKind::Newline {
                break;
            }
            replacement.push(token.to_owned());
        }
        let (start, end) = line
            .offset_range()
            .context("Define directive in illegal location")?;
        // TODO: handle redefines properly
        self.insert(
            key.text.to_string(),
            Definition {
                replacement,
                args,
                provenance: Provenance::FileDefined {
                    start,
                    end,
                    path: path.to_path_buf(),
                },
            },
        );
        Ok(())
    }

    pub fn handle_undef(&mut self, line: &LogicalLine) -> anyhow::Result<()> {
        // Expect possible whitespace, then 'undef'.
        let mut iter = CppTokenIterator::new(&line.text);
        iter.consume_whitespace();
        let directive = iter
            .consume_directive()
            .context("Expected undef directive")?;
        if directive.kind != CppTokenKind::Directive(CppDirectiveKind::Undef) {
            return Err(anyhow!("Expected undef directive"));
        }
        // Expect whitespace, then an identifier
        let _ = iter.consume_whitespace().context("Expected whitespace")?;
        let key = iter.consume_identifier().context("Expected identifier")?;
        // Expect nothing else on line
        iter.consume_whitespace();
        let _ = iter
            .consume_newline()
            .context("Malformed undef directive")?;
        self.remove(key.text)
            .context(format!("Cannot undef undefined identifier: {}", key.text))?;
        Ok(())
    }

    pub fn handle_ifdef(&mut self, line: &LogicalLine) -> anyhow::Result<bool> {
        // Expect possible whitespace, then 'ifdef'.
        let mut iter = CppTokenIterator::new(&line.text);
        iter.consume_whitespace();
        let directive = iter
            .consume_directive()
            .context("Expected ifdef directive")?;
        if directive.kind != CppTokenKind::Directive(CppDirectiveKind::Ifdef) {
            return Err(anyhow!("Expected ifdef directive"));
        }
        // Expect whitespace, then an identifier
        let _ = iter.consume_whitespace().context("Expected whitespace")?;
        let key = iter.consume_identifier().context("Expected identifier")?;
        // Expect possible whitespace, then newline
        iter.consume_whitespace();
        let _ = iter.consume_newline().context("Malformed ifdef")?;
        Ok(self.contains_key(key.text))
    }

    pub fn handle_ifndef(&mut self, line: &LogicalLine) -> anyhow::Result<bool> {
        // TODO combine with handle_ifdef, reduce repeat code
        // Expect possible whitespace, then 'ifdef'.
        let mut iter = CppTokenIterator::new(&line.text);
        iter.consume_whitespace();
        let directive = iter
            .consume_directive()
            .context("Expected ifndef directive")?;
        if directive.kind != CppTokenKind::Directive(CppDirectiveKind::Ifndef) {
            return Err(anyhow!("Expected ifndef directive"));
        }
        // Expect whitespace, then an identifier
        let _ = iter.consume_whitespace().context("Expected whitespace")?;
        let key = iter.consume_identifier().context("Expected identifier")?;
        // Expect possible whitespace, then newline
        iter.consume_whitespace();
        let _ = iter.consume_newline().context("Malformed ifndef")?;
        Ok(!self.contains_key(key.text))
    }
}

/// a C preprocessor implementation that preserves provenance information
/// when expanding macros and includes.
pub struct CPreprocessor {
    #[allow(dead_code)]
    path: PathBuf,
    snippets: Snippets,
    #[allow(dead_code)]
    defines: Definitions,
}

impl CPreprocessor {
    pub fn new(
        input: &SourceCode,
        path: PathBuf,
        user_defines: &HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let mut defines = Definitions::new();
        let datetime: DateTime<Local> = Local::now();
        // Format date as "Mmm dd yyyy", e.g. "Jan 19 2024". Date number is space-padded.
        let date = CppToken {
            text: format!("\"{}\"", datetime.format("%b %e %Y")),
            kind: CppTokenKind::String,
        };
        defines.insert(
            "__DATE__".to_string(),
            Definition {
                replacement: vec![date],
                args: None,
                provenance: Provenance::SystemDefined,
            },
        );
        // Format time as "hh:mm:ss", e.g. "13:45:30". Time numbers are zero-padded.
        let time = CppToken {
            text: format!("\"{}\"", datetime.format("%H:%M:%S")),
            kind: CppTokenKind::String,
        };
        defines.insert(
            "__TIME__".to_string(),
            Definition {
                replacement: vec![time],
                args: None,
                provenance: Provenance::SystemDefined,
            },
        );
        // Add user definitions from command line.
        // Must come after __DATE__ and __TIME__ to allow them to be overwritten.
        for (key, value) in user_defines {
            let replacement = CppTokenIterator::new(value)
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            defines.insert(
                key.clone(),
                Definition {
                    replacement,
                    args: None,
                    provenance: Provenance::UserDefined,
                },
            );
        }
        // Preprocess the input.
        let mut snippets = Snippets::new();
        let mut if_stack = IfStack::new();
        for line in LogicalLines::new(input) {
            if let Some((_, directive)) = regex_captures!(r"\s*#\s*([a-z]+)", &line.text) {
                match directive {
                    "define" => {
                        if if_stack.is_clean() {
                            defines.handle_define(&line, &path)?;
                        }
                    }
                    "undef" => {
                        if if_stack.is_clean() {
                            defines.handle_undef(&line)?;
                        }
                    }
                    "ifdef" => {
                        if_stack.push(defines.handle_ifdef(&line)?);
                    }
                    "ifndef" => {
                        if_stack.push(defines.handle_ifndef(&line)?);
                    }
                    "else" => {
                        // Expect possible whitespace, then 'else', possible whitespace, then newline
                        let mut iter = CppTokenIterator::new(&line.text);
                        iter.consume_whitespace();
                        let token = iter
                            .consume_directive()
                            .context("Expected else directive")?;
                        if token.kind != CppTokenKind::Directive(CppDirectiveKind::Else) {
                            return Err(anyhow!("Expected else directive"));
                        }
                        iter.consume_whitespace();
                        let _ = iter
                            .consume_newline()
                            .context("Else directive should be on empty line")?;
                        if_stack
                            .toggle()
                            .ok_or_else(|| anyhow!("Encountered unexpected else directive"))?;
                    }
                    "endif" => {
                        // Expect possible whitespace, then 'endif', possible whitespace, then newline
                        let mut iter = CppTokenIterator::new(&line.text);
                        iter.consume_whitespace();
                        let token = iter
                            .consume_directive()
                            .context("Expected endif directive")?;
                        if token.kind != CppTokenKind::Directive(CppDirectiveKind::Endif) {
                            return Err(anyhow!("Expected endif directive"));
                        }
                        iter.consume_whitespace();
                        let _ = iter
                            .consume_newline()
                            .context("Endif directive should be on empty line")?;
                        if_stack
                            .pop()
                            .ok_or_else(|| anyhow!("Encountered unexpected endif directive"))?;
                    }
                    _ => {
                        // Unknown directive, ignore line
                    }
                }
            } else {
                // Not a pre-processor directive line
                let mut iter = CppTokenIterator::new(&line.text);
                while let Some(token) = iter.next() {
                    match token.kind {
                        CppTokenKind::Identifier => {
                            if !if_stack.is_clean() {
                                continue;
                            }
                            match defines.macro_kind(token.text) {
                                MacroKind::Function => {
                                    // Try to parse argument list
                                    if let Some(arglist) = iter.consume_arglist_invocation()? {
                                        let (definition, replacement) =
                                            defines.expand_function_macro(token.text, &arglist)?;
                                        snippets.push(&replacement, definition.provenance.clone());
                                        continue;
                                    } else {
                                        // No argument list, treat as normal identifier
                                    }
                                }
                                MacroKind::Object => {
                                    let (definition, replacement) =
                                        defines.expand_object_macro(token.text)?;
                                    snippets.push(&replacement, definition.provenance.clone());
                                    continue;
                                }
                                MacroKind::None => {
                                    // Not a macro, handle special cases and plain identifiers below
                                }
                            }
                            if token.text == "__LINE__" {
                                // Get the line number of the start of this token, accounting
                                // for line continuations.
                                let real_offset = line
                                    .offset(token.start)
                                    .context("__LINE__ directive in illegal location")?;
                                let real_offset = TextSize::from(real_offset as u32);
                                let real_line = input.line_index(real_offset).to_string();
                                snippets.push(&real_line, Provenance::SystemDefined);
                            } else if token.text == "__FILE__" {
                                snippets.push(
                                    &format!("\"{}\"", path.to_string_lossy()),
                                    Provenance::SystemDefined,
                                );
                            } else {
                                let start = line
                                    .offset(token.start)
                                    .context("Token in illegal location")?;
                                let end = line
                                    .offset(token.end)
                                    .context("Token in illegal location")?;
                                snippets.push(token.text, Provenance::LocalText { start, end });
                            }
                        }
                        CppTokenKind::Directive(kind) => {
                            // Should not happen, directives are handled above.
                            return Err(anyhow!(
                                "Unexpected directive token {:?} in non-directive line",
                                kind
                            ));
                        }
                        _ => {
                            if !if_stack.is_clean() {
                                continue;
                            }
                            let start = line
                                .offset(token.start)
                                .context("Token in illegal location")?;
                            let end = line
                                .offset(token.end)
                                .context("Token in illegal location")?;
                            snippets.push(token.text, Provenance::LocalText { start, end });
                        }
                    }
                }
            }
        }
        Ok(Self {
            path,
            snippets,
            defines,
        })
    }

    pub fn output(&self) -> String {
        self.snippets.collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dedent::dedent;
    use lazy_regex::regex_is_match;
    use ruff_source_file::LineIndex;

    fn preprocess(code: &str) -> anyhow::Result<(String, Vec<Snippet>)> {
        let line_index = LineIndex::from_source_text(code);
        let code = SourceCode::new(code, &line_index);
        let mut user_defines = HashMap::new();
        user_defines.insert("__GNU__".to_string(), "".to_string());
        user_defines.insert("TEST".to_string(), 42.to_string());
        let preprocessor = CPreprocessor::new(&code, PathBuf::from("test.f90"), &user_defines)?;
        Ok((preprocessor.output(), preprocessor.snippets.inner))
    }

    #[test]
    fn test_datetime() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            __DATE__
            __TIME__ UTC
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let output = output.replace('\n', " ");
        // Check that output matches the expected datetime format
        assert!(regex_is_match!(
            r#""[A-z]{3} [\s\d][\d] \d{4}" "\d{2}:\d{2}:\d{2}""#,
            &output
        ));
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 4);
        // __DATE__
        assert_eq!(snippets[0].provenance, Provenance::SystemDefined);
        // \n
        assert_eq!(
            snippets[1].provenance,
            Provenance::LocalText { start: 8, end: 9 }
        );
        // __TIME__
        assert_eq!(snippets[2].provenance, Provenance::SystemDefined);
        // " UTC"
        assert_eq!(
            snippets[3].provenance,
            Provenance::LocalText { start: 17, end: 21 }
        );
        Ok(())
    }

    #[test]
    fn test_object_macros() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define W 5
            #define X 10
            #define Y
            #define Z W,Y X
            #undef X
            program p
              integer :: X
              X = 12
              print *, Z, __FILE__, __LINE__, TEST
            end program p
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            program p
              integer :: X
              X = 12
              print *, 5, X, "test.f90", 9, 42
            end program p
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 9);
        // Z
        if let Provenance::FileDefined { start, end, path } = &snippets[1].provenance {
            assert_eq!(&code[*start..*end], "#define Z W,Y X\n");
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for Z");
        }
        // ", " following Z
        if let Provenance::LocalText { start, end } = &snippets[2].provenance {
            assert_eq!(&code[*start..*end], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following Z");
        }
        // __FILE__
        assert_eq!(snippets[3].provenance, Provenance::SystemDefined);
        // ", " following __FILE__
        if let Provenance::LocalText { start, end } = &snippets[4].provenance {
            assert_eq!(&code[*start..*end], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following __FILE__");
        }
        // __LINE__
        assert_eq!(snippets[5].provenance, Provenance::SystemDefined);
        // ", " following __LINE__
        if let Provenance::LocalText { start, end } = &snippets[6].provenance {
            assert_eq!(&code[*start..*end], ", ");
        } else {
            panic!("Expected LocalText provenance for ', ' following __LINE__");
        }
        // TEST
        assert_eq!(snippets[7].provenance, Provenance::UserDefined);
        // Rest of code
        if let Provenance::LocalText { start, end } = &snippets[8].provenance {
            assert_eq!(&code[*start..*end], "\nend program p");
        } else {
            panic!("Expected LocalText provenance for rest of code");
        }
        Ok(())
    }

    #[test]
    fn test_function_macros() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define W 5
            #define foo( x ) (x + W)
            #define bar(x, y) x//y
            #define baz() 10
            program p
              implicit none
              integer, parameter :: foo = 1
              integer, parameter :: baz = 3
              print *, foo, foo(5), foo(foo + 2), foo(foo(7) + W)
              print *, bar("hello, ","world!")
              print *, baz, baz(), foo(baz())
            end program p
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            program p
              implicit none
              integer, parameter :: foo = 1
              integer, parameter :: baz = 3
              print *, foo, (5 + 5), (foo + 2 + 5), ((7 + 5) + 5 + 5)
              print *, "hello, "//"world!"
              print *, baz, 10, (10 + 5)
            end program p
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 13);
        // foo
        for i in [1, 3, 5, 11] {
            if let Provenance::FileDefined { start, end, path } = &snippets[i].provenance {
                assert_eq!(&code[*start..*end], "#define foo( x ) (x + W)\n");
                assert_eq!(path, &PathBuf::from("test.f90"));
            } else {
                panic!("Expected FileDefined provenance for foo");
            }
        }
        if let Provenance::FileDefined { start, end, path } = &snippets[7].provenance {
            assert_eq!(&code[*start..*end], "#define bar(x, y) x//y\n");
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for bar");
        }
        if let Provenance::FileDefined { start, end, path } = &snippets[9].provenance {
            assert_eq!(&code[*start..*end], "#define baz() 10\n");
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for baz");
        }
        Ok(())
    }

    #[test]
    fn test_nested_function_macros() -> anyhow::Result<()> {
        // change foo(7) to foo(7) + W
        // Add foo(baz())
        let code = dedent!(
            r#"
            #define foo(x, y) (x + y)
            #define bar(x) foo(x, y)
            foo(bar(2), 10)
        "#
        );
        let (output, _) = preprocess(code)?;
        let expected = "((2 +  y) +  10)";
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_if() -> anyhow::Result<()> {
        // Note: definitions are expanded even in Fortran comments!
        let code = dedent!(
            r#"
            #ifdef X
            #  ifndef Y
            ! X !Y
            #    define Z 10
            #  else
            ! X Y
            #    define Z 20
            #  endif
            #else
            #  ifdef Y
            ! !X Y
            #    define Z 30
            #  else
            ! !X !Y
            #    define Z 40
            #  endif
            #endif
            program p
              print *, Z
            end program p
        "#
        );

        let (output, _) = preprocess(&["#define X", code].join("\n"))?;
        let expected = dedent!(
            r#"
            !  !Y
            program p
              print *, 10
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(&["#define X", "#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            !  
            program p
              print *, 20
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(&["#define Y", code].join("\n"))?;
        let expected = dedent!(
            r#"
            ! !X 
            program p
              print *, 30
            end program p
        "#
        );
        assert_eq!(output, expected);

        let (output, _) = preprocess(code)?;
        let expected = dedent!(
            r#"
            ! !X !Y
            program p
              print *, 40
            end program p
        "#
        );
        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn test_unknown_directive() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #define X test
            #unknown_directive
            end
        "#
        );
        let (output, _) = preprocess(code)?;
        assert_eq!(output.as_str(), "end");
        Ok(())
    }

    #[test]
    fn test_line_continuation() -> anyhow::Result<()> {
        let code = dedent!(
            r#"
            #def\
            ine X \
            (1 + \
            2)
            print\
             *, X, __\
            LI\
            NE_\
            _
        "#
        );
        let (output, snippets) = preprocess(code)?;
        let expected = dedent!(
            r#"
            print *, (1 + 2), 6
        "#
        );
        assert_eq!(output, expected);
        // Check that snippets have correct provenance
        assert_eq!(snippets.len(), 4);
        // "print *, ""
        if let Provenance::LocalText { start, end } = &snippets[0].provenance {
            assert_eq!(&code[*start..*end], "print\\\n *, ");
        } else {
            panic!("Expected LocalText provenance for 'print *, '");
        }
        // X
        if let Provenance::FileDefined { start, end, path } = &snippets[1].provenance {
            assert_eq!(&code[*start..*end], "#def\\\nine X \\\n(1 + \\\n2)\n");
            assert_eq!(path, &PathBuf::from("test.f90"));
        } else {
            panic!("Expected FileDefined provenance for X");
        }
        // ", "
        if let Provenance::LocalText { start, end } = &snippets[2].provenance {
            assert_eq!(&code[*start..*end], ", ");
        } else {
            panic!("Expected LocalText provenance for ', '");
        }
        // __LINE__
        assert_eq!(snippets[3].provenance, Provenance::SystemDefined);
        Ok(())
    }
}
