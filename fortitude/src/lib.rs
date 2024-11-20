mod ast;
pub mod check;
pub mod cli;
pub mod explain;
mod fs;
mod message;
mod printer;
mod registry;
mod rule_redirects;
mod rule_selector;
mod rules;
mod settings;
#[cfg(test)]
mod test;
mod text_helpers;
pub use crate::registry::clap_completion::RuleParser;
pub use crate::rule_selector::clap_completion::RuleSelectorParser;
use ast::{parse, FortitudeNode};
use ruff_diagnostics::{Diagnostic, DiagnosticKind};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::{TextRange, TextSize};
use settings::Settings;
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

/// Simplify making a `SourceFile` in tests
#[cfg(test)]
pub fn test_file(source: &str) -> SourceFile {
    use ruff_source_file::SourceFileBuilder;
    use textwrap::dedent;

    SourceFileBuilder::new("test.f90", dedent(source)).finish()
}
