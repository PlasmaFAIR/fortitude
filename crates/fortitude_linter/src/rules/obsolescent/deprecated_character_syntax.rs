use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What does it do?
/// Checks for outdated declarations of `character*N`, `character*(*)`,
/// `character*(:)`, and `character*(integer-expression)`.
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
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // Rule only applies to `character`.
        // Expect child(0) to always be present.
        let dtype = node.child(0)?;
        if dtype.kind() != "character" {
            return None;
        }

        // If 'kind' field isn't present, exit early
        let kind = node.child_by_field_name("kind")?;
        let kind_text = kind.text();

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
            kind.named_child(1)?.text().to_string()
        } else {
            child.text().to_string()
        };

        let original = node.text().to_string();
        let dtype = dtype.text().to_string();
        let replacement = format!("{dtype}(len={length})");
        let fix = Fix::safe_edit(node.edit_replacement(replacement));
        some_vec![
            context
                .create_diagnostic(
                    Self {
                        original,
                        dtype,
                        length
                    },
                    node
                )
                .with_fix(fix)
        ]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["intrinsic_type"]
    }
}
