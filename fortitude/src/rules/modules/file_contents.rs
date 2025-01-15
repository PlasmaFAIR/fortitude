use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for multiple modules in one file
///
/// ## Why is this bad?
/// Placing each module into its own file improves maintainability
/// by making each module easier to locate for developers, and also
/// making dependency generation in build systems easier.
#[violation]
pub struct MultipleModules {}

impl Violation for MultipleModules {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Multiple modules in one file, split into one module per file")
    }
}

impl AstRule for MultipleModules {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let violations: Vec<Diagnostic> = node
            .children(&mut node.walk())
            .filter(|node| node.kind() == "module")
            .skip(1)
            .map(|m| -> Diagnostic {
                let m_first = m.child(0).unwrap_or(m);
                Diagnostic::from_node(MultipleModules {}, &m_first)
            })
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["translation_unit"]
    }
}

/// ## What it does
/// Checks for programs and modules in one file
///
/// ## Why is this bad?
/// Separating top-level constructs into their own files improves
/// maintainability by making each easier to locate for developers,
/// and also making dependency generation in build systems easier.
#[violation]
pub struct ProgramWithModule {}

impl Violation for ProgramWithModule {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Program and module in one file, split into their own files")
    }
}

impl AstRule for ProgramWithModule {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // Mark the violation on the second, and subsequent, occurences
        let violations: Vec<Diagnostic> = node
            .children(&mut node.walk())
            .filter(|node| node.kind() == "module" || node.kind() == "program")
            .skip(1)
            .map(|m| -> Diagnostic {
                let m_first = m.child(0).unwrap_or(m);
                Diagnostic::from_node(ProgramWithModule {}, &m_first)
            })
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["translation_unit"]
    }
}
