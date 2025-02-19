use crate::ast::FortitudeNode;
use crate::rules::utilities::literal_as_io_unit;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for the literals `5` or `6` as units in `read`/`write` statements.
///
/// ## Why is this bad?
/// The Fortran standard does not specify numeric values for `stdin` or
/// `stdout`. Instead, use the named constants `input_unit` and `output_unit`
/// from the `iso_fortran_env` module.
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableIoUnit {
    value: i32,
    kind: String,
    replacement: Option<String>,
}

impl Violation for NonPortableIoUnit {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { value, kind, .. } = self;
        format!("Non-portable unit '{value}' in '{kind}' statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { replacement, .. } = self;
        replacement
            .as_ref()
            .map(|replacement| format!("Use `{replacement}` from `iso_fortran_env`"))
    }
}

impl AstRule for NonPortableIoUnit {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let unit = literal_as_io_unit(node, src)?;

        let value = unit
            .to_text(src.source_text())?
            .parse::<i32>()
            .unwrap_or_default();
        let is_read = node.kind() == "read_statement";
        let is_write = node.kind() == "write_statement";

        let kind = if is_read { "read" } else { "write" }.to_string();

        let replacement = if is_read && value == 5 {
            Some("input_unit".to_string())
        } else if is_write && value == 6 {
            Some("output_unit".to_string())
        } else {
            None
        };

        some_vec!(Diagnostic::from_node(
            Self {
                value,
                kind,
                replacement
            },
            &unit
        ))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["read_statement", "write_statement"]
    }
}
