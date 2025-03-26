pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod allow_comments;
mod ast;
pub mod check;
pub mod cli;
mod configuration;
mod diagnostics;
pub mod explain;
mod fix;
mod fs;
pub mod locator;
pub mod logging;
pub mod message;
pub mod options;
pub mod options_base;
mod printer;
pub mod registry;
mod rule_redirects;
mod rule_selector;
pub mod rule_table;
pub mod rules;
pub mod settings;
mod show_files;
mod show_settings;
pub mod stdin;
#[cfg(test)]
mod test;
mod text_helpers;
pub mod version;
pub use crate::registry::clap_completion::RuleParser;
pub use crate::rule_selector::clap_completion::RuleSelectorParser;

use ruff_diagnostics::{Diagnostic, DiagnosticKind};
use ruff_source_file::SourceFile;
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
}

/// Simplify making a `SourceFile` in tests
#[cfg(test)]
pub fn test_file(source: &str) -> SourceFile {
    use ruff_source_file::SourceFileBuilder;
    use textwrap::dedent;

    SourceFileBuilder::new("test.f90", dedent(source)).finish()
}
