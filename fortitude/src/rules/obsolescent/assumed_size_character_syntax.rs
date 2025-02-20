use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for deprecated declarations of `character`
///
/// ## Why is this bad?
/// The syntax `character*(*)` is a deprecated form of `character(len=*)`. Prefer the
/// second form.
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedAssumedSizeCharacter {
    name: String,
}

impl Violation for DeprecatedAssumedSizeCharacter {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("character '{name}' uses deprecated syntax for assumed size")
    }
}

impl AstRule for DeprecatedAssumedSizeCharacter {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        let declaration = node
            .ancestors()
            .find(|parent| parent.kind() == "variable_declaration")?;

        // Only applies to `character`
        if declaration.parse_intrinsic_type()?.to_lowercase() != "character" {
            return None;
        }

        // Are we immediately (modulo whitespace) in front of `(...)`?
        if node.next_sibling()?.kind() != "(" {
            return None;
        }

        // Collect all declarations on this line
        let all_decls = declaration
            .children_by_field_name("declarator", &mut declaration.walk())
            .filter_map(|declarator| {
                let identifier = match declarator.kind() {
                    "identifier" => Some(declarator),
                    "sized_declarator" => declarator.child_with_name("identifier"),
                    _ => None,
                }?;
                identifier.to_text(src)
            })
            .map(|name| name.to_string())
            .map(|name| Diagnostic::from_node(Self { name }, node))
            .collect_vec();

        Some(all_decls)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["assumed_size"]
    }
}
