use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use itertools::Itertools;
use lazy_regex::regex_captures;
use ruff_diagnostics::{Diagnostic, Fix, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for outdated declarations of `character*N`
///
/// ## Why is this bad?
/// The syntax `character*N` has been replaced by `character(len=N)` in modern
/// Fortran. Prefer the second form.
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedCharacterSyntax {
    length: String,
}

impl Violation for DeprecatedCharacterSyntax {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { length } = self;
        format!("'character*{length}' uses outdated syntax")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { length, .. } = self;
        Some(format!("Replace with 'character(len={length})'"))
    }
}

impl AstRule for DeprecatedCharacterSyntax {
    fn check(
        _settings: &Settings,
        node: &Node,
        source_file: &SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let src = source_file.source_text();

        // Rule only applies to `character`.
        // Expect child(0) to always be present.
        if node.child(0)?.kind().to_lowercase() != "character" {
            return None;
        }

        // If 'kind' field isn't present, exit early
        let kind = node.child_by_field_name("kind")?;
        let kind_text = kind.to_text(src)?;

        // If 'kind' field doesn't match the regex, exit early
        let (_, length) = regex_captures!(r#"\*\s*(\d*)"#, kind_text)?;

        let replacement = format!("character(len={})", length);
        let fix = Fix::safe_edit(node.edit_replacement(source_file, replacement));
        some_vec![Diagnostic::from_node(
            Self {
                length: length.to_string()
            },
            node
        )
        .with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

/// ## What does it do?
/// Checks for deprecated declarations of `character*(*)`
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
