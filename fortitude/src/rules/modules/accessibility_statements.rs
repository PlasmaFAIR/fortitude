use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for missing `private` or `public` accessibility statements in modules
///
/// ## Why is this bad?
/// The `private` statement makes all entities (variables, types, procedures)
/// private by default, requiring an explicit `public` attribute to make them
/// available. As well as improving encapsulation between modules, this also
/// makes it possible to detect unused entities.
///
/// A `public` statement in a module does not change the default behaviour,
/// and therefore all entities will be available from outside the module
/// unless they are individually given a `private` attribute. This brings
/// all of the same downsides as the default behaviour, but an explicit
/// `public` statement makes it clear that the programmer is choosing
/// this behaviour intentionally.  
#[violation]
pub struct MissingAccessibilityStatement {
    name: String,
}

impl Violation for MissingAccessibilityStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "module '{}' missing default accessibility statement",
            self.name
        )
    }
}

impl AstRule for MissingAccessibilityStatement {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let module = node.parent()?;

        let bare_private_statement = match module.child_with_name("private_statement") {
            Some(statement) => statement.named_child(0).is_none(),
            None => false,
        };

        let bare_public_statement = match module.child_with_name("public_statement") {
            Some(statement) => statement.named_child(0).is_none(),
            None => false,
        };

        // No statement whatsoever
        if !bare_private_statement && !bare_public_statement {
            let name = node.named_child(0)?.to_text(src.source_text())?.to_string();
            return some_vec![Diagnostic::from_node(
                MissingAccessibilityStatement { name },
                node
            )];
        }

        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module_statement"]
    }
}

/// ## What it does
/// Checks if the default accessibility in modules is set to `public`
///
/// ## Why is this bad?
/// The `public` statement makes all entities (variables, types, procedures)
/// public by default. This decreases encapsulation and makes it more likely to
/// accidentally expose more than necessary. Public accessibility also makes
/// it harder to detect unused entities, which can often be indicative of
/// errors within the code.
#[violation]
pub struct DefaultPublicAccessibility {
    name: String,
}

impl Violation for DefaultPublicAccessibility {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("module '{}' has default `public` accessibility", self.name)
    }
}

impl AstRule for DefaultPublicAccessibility {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let module = node.parent()?;

        let public_statement = module.child_with_name("public_statement")?;

        // Bare `public` statement`
        if public_statement.named_child(0).is_none() {
            let name = node.named_child(0)?.to_text(src.source_text())?.to_string();
            return some_vec![Diagnostic::from_node(
                DefaultPublicAccessibility { name },
                node
            )];
        }

        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module_statement"]
    }
}
