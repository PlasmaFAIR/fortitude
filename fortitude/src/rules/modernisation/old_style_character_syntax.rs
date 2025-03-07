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
pub(crate) struct OldStyleCharacterSyntax {
    length: String,
}

impl Violation for OldStyleCharacterSyntax {
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

impl AstRule for OldStyleCharacterSyntax {
    fn check(
        _settings: &Settings,
        node: &Node,
        source_file: &SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let src = source_file.source_text();

        // Only applies to `character`
        if node.child(0)?.kind().to_lowercase() != "character" {
            return None;
        }

        if let Some(kind) = node.child_by_field_name("kind") {
            let kind_text = kind.to_text(src)?;
            if let Some((_, length)) = regex_captures!(r#"\*\s*(\d*)"#, kind_text) {
                let replacement = format!("character(len={})", length);
                let fix = Fix::safe_edit(node.edit_replacement(source_file, replacement));
                return some_vec![Diagnostic::from_node(
                    Self {
                        length: length.to_string()
                    },
                    node
                )
                .with_fix(fix)];
            }
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
