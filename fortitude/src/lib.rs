mod ast;
pub mod check;
pub mod cli;
pub mod explain;
mod registry;
mod rule_redirects;
mod rule_selector;
mod rules;
mod settings;
#[cfg(test)]
mod test;
pub use crate::registry::clap_completion::RuleParser;
use crate::registry::AsRule;
pub use crate::rule_selector::clap_completion::RuleSelectorParser;
use annotate_snippets::{Level, Renderer, Snippet};
use ast::{parse, FortitudeNode};
use colored::{ColoredString, Colorize};
use ruff_diagnostics::{Diagnostic, DiagnosticKind, Fix};
use ruff_source_file::{OneIndexed, SourceFile, SourceLocation};
use ruff_text_size::{Ranged, TextRange, TextSize};
use settings::Settings;
use std::cmp::Ordering;
use std::fmt;
use std::path::Path;
use tree_sitter::Node;

// Violation type
// --------------

pub trait FromAstNode {
    fn from_node<T: Into<DiagnosticKind>>(violation: T, node: &Node) -> Self;
}

impl FromAstNode for Diagnostic {
    fn from_node<T: Into<DiagnosticKind>>(violation: T, node: &Node) -> Self {
        Self::new(
            violation,
            TextRange::new(
                TextSize::try_from(node.start_byte()).unwrap(),
                TextSize::try_from(node.end_byte()).unwrap(),
            ),
        )
    }
}

pub trait FromStartEndLineCol {
    /// Create new `Violation` from zero-index start/end line/column numbers
    fn from_start_end_line_col<T: Into<DiagnosticKind>>(
        kind: T,
        source: &SourceFile,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self;
}

impl FromStartEndLineCol for Diagnostic {
    fn from_start_end_line_col<T: Into<DiagnosticKind>>(
        kind: T,
        source: &SourceFile,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        let source_code = source.to_source_code();
        let start_line_offset = source_code.line_start(OneIndexed::from_zero_indexed(start_line));
        let start_offset = start_line_offset + TextSize::try_from(start_col).unwrap();
        let end_line_offset = source_code.line_start(OneIndexed::from_zero_indexed(end_line));
        let end_offset = end_line_offset + TextSize::try_from(end_col).unwrap();
        Diagnostic::new(kind, TextRange::new(start_offset, end_offset))
    }
}

// Rule trait
// ----------

/// Implemented by rules that act directly on the file path.
pub trait PathRule {
    fn check(settings: &Settings, path: &Path) -> Option<Diagnostic>;
}

/// Implemented by rules that analyse lines of code directly, using regex or otherwise.
pub trait TextRule {
    fn check(settings: &Settings, source: &SourceFile) -> Vec<Diagnostic>;
}

/// Implemented by rules that analyse the abstract syntax tree.
pub trait AstRule {
    fn check(settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>>;

    /// Return list of tree-sitter node types on which a rule should trigger.
    fn entrypoints() -> Vec<&'static str>;

    /// Apply a rule over some text, generating all violations raised as a result.
    fn apply(source: &SourceFile) -> anyhow::Result<Vec<Diagnostic>> {
        let entrypoints = Self::entrypoints();
        Ok(parse(source.source_text())?
            .root_node()
            .named_descendants()
            .filter(|x| entrypoints.contains(&x.kind()))
            .filter_map(|x| Self::check(&Settings::default(), &x, source))
            .flatten()
            .collect())
    }
}

// Violation diagnostics
// ---------------------

/// Reports of each violation. They are pretty-printable and sortable.
#[derive(Debug, PartialEq, Eq)]
pub struct DiagnosticMessage<'a> {
    kind: DiagnosticKind,
    range: TextRange,
    /// The file where an error was reported.
    file: &'a SourceFile,
    /// The rule code that was violated, expressed as a string.
    code: String,
    /// The suggested fix for the violation.
    fix: Option<Fix>,
}

impl<'a> DiagnosticMessage<'a> {
    pub fn from_ruff(file: &'a SourceFile, diagnostic: Diagnostic) -> Self {
        let code = diagnostic.kind.rule().noqa_code().to_string();
        Self {
            kind: diagnostic.kind,
            file,
            code,
            range: diagnostic.range,
            fix: diagnostic.fix,
        }
    }
}

impl<'a> Ord for DiagnosticMessage<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.file, self.range.start()).cmp(&(other.file, other.range.start()))
    }
}

impl<'a> PartialOrd for DiagnosticMessage<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ranged for DiagnosticMessage<'a> {
    fn range(&self) -> TextRange {
        self.range
    }
}

impl<'a> fmt::Display for DiagnosticMessage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut path: ColoredString = self.file.name().bold();
        let mut code: ColoredString = self.code.bold().bright_red();

        // Disable colours for tests, if the user requests it via env var, or non-tty
        if cfg!(test) || !colored::control::SHOULD_COLORIZE.should_colorize() {
            path = path.clear();
            code = code.clear();
        };

        let message = self.kind.body.as_str();
        let suggestion = &self.kind.suggestion;
        if self.range != TextRange::default() {
            format_violation(self, f, &self.range, message, suggestion, &path, &code)
        } else {
            write!(f, "{path}: {code} {message}")
        }
    }
}

fn format_violation(
    diagnostic: &DiagnosticMessage,
    f: &mut fmt::Formatter,
    range: &TextRange,
    message: &str,
    suggestion: &Option<String>,
    path: &ColoredString,
    code: &ColoredString,
) -> fmt::Result {
    let source_code = diagnostic.file.to_source_code();
    let content_start_index = source_code.line_index(range.start());
    let mut start_index = content_start_index.saturating_sub(2);

    // Trim leading empty lines.
    while start_index < content_start_index {
        if !source_code.line_text(start_index).trim().is_empty() {
            break;
        }
        start_index = start_index.saturating_add(1);
    }

    let content_end_index = source_code.line_index(range.end());
    let mut end_index = content_end_index
        .saturating_add(2)
        .min(OneIndexed::from_zero_indexed(source_code.line_count()));

    // Trim following empty lines.
    while end_index > content_end_index {
        if !source_code.line_text(end_index).trim().is_empty() {
            break;
        }
        end_index = end_index.saturating_sub(1);
    }

    let start_offset = source_code.line_start(start_index);
    let end_offset = source_code.line_end(end_index);

    let source = source_code.slice(TextRange::new(start_offset, end_offset));
    let message_range = range - start_offset;

    let start_char = source[TextRange::up_to(message_range.start())]
        .chars()
        .count();
    let end_char = source[TextRange::up_to(message_range.end())]
        .chars()
        .count();

    // Some annoyance here: we *have* to have some level prefix to our
    // message. Might be fixed in future version of annotate-snippets
    // -- or we use an earlier version with more control.
    // Also, we could use `.origin(path)` to get the filename and
    // line:col automatically, but there is currently a bug in the
    // library when annotating the start of the line
    let SourceLocation { row, column } = source_code.source_location(range.start());
    let message_line = format!("{path}:{row}:{column}: {code} {message}");
    let snippet = Level::Warning.title(&message_line).snippet(
        Snippet::source(source)
            .line_start(start_index.get())
            .annotation(Level::Error.span(start_char..end_char).label(code)),
    );

    let snippet_with_footer = if let Some(s) = suggestion {
        snippet.footer(Level::Help.title(s))
    } else {
        snippet
    };

    // Disable colours for tests, if the user requests it via env var, or non-tty
    let renderer = if !cfg!(test) && colored::control::SHOULD_COLORIZE.should_colorize() {
        Renderer::styled()
    } else {
        Renderer::plain()
    };
    let source_block = renderer.render(snippet_with_footer);
    writeln!(f, "{source_block}")?;

    Ok(())
}

/// Simplify making a `SourceFile` in tests
#[cfg(test)]
pub fn test_file(source: &str) -> SourceFile {
    use ruff_source_file::SourceFileBuilder;
    use textwrap::dedent;

    SourceFileBuilder::new("test.f90", dedent(source)).finish()
}
