use crate::Provenance;
use crate::logical_lines::LogicalLine;
use crate::tokens::{CppDirectiveKind, CppToken, CppTokenIterator, CppTokenKind};
use anyhow::{Context, anyhow};
use std::collections::HashMap;
use std::path::Path;

/// An object macro definition.
pub struct Definition {
    replacement: Vec<CppToken>,
    args: Option<Vec<String>>,
    provenance: Provenance,
}

impl Definition {
    pub fn new(replacement: &[CppToken], args: Option<&[String]>, provenance: Provenance) -> Self {
        Definition {
            replacement: replacement.to_vec(),
            args: args.map(|a| a.to_vec()),
            provenance,
        }
    }

    pub fn replacement(&self) -> &Vec<CppToken> {
        &self.replacement
    }

    pub fn args(&self) -> Option<&Vec<String>> {
        self.args.as_ref()
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
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
        // Due to the removal of comments in replacement text, it is possible
        // to encounter
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
            // Skip any comments in the replacement text.
            // Necessary to handle concatenation properly.
            if token.kind == CppTokenKind::Comment {
                continue;
            }
            result.push_str(&token.text);
        }
        Ok(result)
    }

    pub fn expand_object_macro(&self, key: &str) -> anyhow::Result<(&Definition, String)> {
        let definition = self.get(key).context("Internal: Macro not defined")?;
        let result = self.expand_macro_tokens(definition.replacement())?;
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
            .args()
            .context("Internal: Expected function macro argument list")?;
        if def_args.len() != args.len() {
            return Err(anyhow!(
                "Function macro argument count mismatch, {key}, {def_args:?}, {args:?}"
            ));
        }
        // Perform substitutions on first pass
        let mut substituted = Vec::new();
        for token in definition.replacement() {
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
        if let Some(definition) = self.get(key) {
            if definition.args().is_some() {
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
        let mut iter = CppTokenIterator::new(line.text());
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
        let (start, end) = line.offset_range();
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
        let mut iter = CppTokenIterator::new(line.text());
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
        let mut iter = CppTokenIterator::new(line.text());
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
        let mut iter = CppTokenIterator::new(line.text());
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
