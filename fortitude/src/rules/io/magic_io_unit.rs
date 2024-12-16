use crate::ast::{is_keyword_argument, FortitudeNode};
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for literal integers as units in IO statements.
///
/// ## Why is this bad?
/// Hardcoding unit numbers makes programs more brittle as it becomes harder to
/// verify units have been opened before reading/writing. Instead, units should
/// be passed in to procedures as arguments, or the `newunit=` argument used for
/// `open` statements.
///
/// Bad:
/// ```f90
/// open(10, file="example.txt", action="read")
/// read(10, fmt=*) int
/// close(10)
/// ```
///
/// Good:
/// ```f90
/// open(newunit=example_unit, file="example.txt", action="read")
/// read(example_unit, fmt=*) int
/// close(example_unit)
/// ```
#[violation]
pub struct MagicIoUnit {
    value: i32,
}

impl Violation for MagicIoUnit {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { value, .. } = self;
        format!("Magic unit '{value}' in IO statement")
    }

    fn fix_title(&self) -> Option<String> {
        Some("Replace with named variable".to_string())
    }
}

impl AstRule for MagicIoUnit {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let unit = literal_as_unit(node, src)?;

        let value = unit
            .to_text(src.source_text())?
            .parse::<i32>()
            .unwrap_or_default();

        some_vec!(Diagnostic::from_node(Self { value }, &unit))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![
            "read_statement",
            "write_statement",
            "open_statement",
            "close_statement",
        ]
    }
}

/// ## What it does
/// Checks for the literals `5` or `6` as units in `read`/`write` statements.
///
/// ## Why is this bad?
/// The Fortran standard does not specify numeric values for `stdin` or
/// `stdout`. Instead, use the named constants `input_unit` and `output_unit`
/// from the `iso_fortran_env` module.
#[violation]
pub struct NonPortableIoUnit {
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
        let unit = literal_as_unit(node, src)?;

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

fn literal_as_unit<'a>(node: &'a Node, src: &SourceFile) -> Option<Node<'a>> {
    let unit = if let Some(unit) = node.child_with_name("unit_identifier") {
        unit.child(0)?
    } else {
        node.named_children(&mut node.walk())
            .find(|child| is_keyword_argument(child, "unit", src.source_text()))
            .map(|node| node.child_by_field_name("value"))??
    };

    if unit.kind() == "number_literal" {
        Some(unit)
    } else {
        None
    }
}
