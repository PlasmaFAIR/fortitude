use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
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
    original: String,
    length: String,
}

impl Violation for DeprecatedCharacterSyntax {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { original, .. } = self;
        format!("'{original}' uses outdated syntax")
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

        // Look for a kind that begins with a '*' and is followed by a length.
        // The length itself might be within any number of parentheses, and may
        // be either a number literal or any number of alphanumeric or
        // underscore characters (by the standard it should be just a number
        // literal, but preprocessor macros may be used with some compilers).
        // Alternatively, the initial '*' may be followed by '(*)' with any
        // amount of whitespace between each character. In this case there
        // must strictly be only one set of parentheses.
        // If 'kind' field doesn't match the regex, exit early
        let (_, length, star) = regex_captures!(
            r#"^\*(?:[\s\(]*([[:word:]]+)[\s\)]*|\s*\(\s*(\*)\s*\))$"#,
            kind_text
        )?;
        // Only one of length or star should be present. The other will be empty.
        let length = if length.is_empty() { star } else { length };

        let original = node.to_text(src)?;
        let replacement = format!("character(len={})", length);
        let fix = Fix::safe_edit(node.edit_replacement(source_file, replacement));
        some_vec![Diagnostic::from_node(
            Self {
                original: original.to_string(),
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
