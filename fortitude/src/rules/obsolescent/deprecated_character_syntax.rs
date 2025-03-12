use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for outdated declarations of `character*N`, 'character*(*)',
/// `character*(:)`, and 'character*(integer-expression)'.
///
/// ## Why is this bad?
/// The syntax `character*N` has been replaced by `character(len=N)` in modern
/// Fortran. Prefer the second form.
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedCharacterSyntax {
    original: String,
    dtype: String,
    length: String,
}

impl AlwaysFixableViolation for DeprecatedCharacterSyntax {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { original, .. } = self;
        format!("'{original}' uses deprecated syntax")
    }

    fn fix_title(&self) -> String {
        let Self { dtype, length, .. } = self;
        format!("Replace with '{dtype}(len={length})'")
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
        let dtype = node.child(0)?;
        if dtype.kind() != "character" {
            return None;
        }

        // If 'kind' field isn't present, exit early
        let kind = node.child_by_field_name("kind")?;
        let kind_text = kind.to_text(src)?;

        // If kind does not start with '*', exit early
        if !kind_text.starts_with('*') {
            return None;
        }

        // The '*' should be followed by:
        // - An integer literal
        // - '(*)'
        // - An integer expression within parentheses
        // For the first case, the first child_node will be a number_literal.
        // For the latter two, the first child node will be `assumed_size`, and
        // the second child node will be the length (which may be a
        // number_literal, a math_expression, or another assumed_size).
        let child = kind.named_child(0)?;
        let length = if child.kind() == "assumed_size" {
            kind.named_child(1)?.to_text(src)?.to_string()
        } else {
            child.to_text(src)?.to_string()
        };

        let original = node.to_text(src)?.to_string();
        let dtype = dtype.to_text(src)?.to_string();
        let replacement = format!("{}(len={})", dtype, length);
        let fix = Fix::safe_edit(node.edit_replacement(source_file, replacement));
        some_vec![Diagnostic::from_node(
            Self {
                original,
                dtype,
                length
            },
            node
        )
        .with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
