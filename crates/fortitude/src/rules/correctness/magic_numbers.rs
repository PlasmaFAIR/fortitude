use crate::ast::FortitudeNode;
use crate::rules::utilities::literal_as_io_unit;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of literals when specifying array sizes
///
/// ## Why is this bad?
/// Prefer named constants to literal integers when declaring arrays. This makes
/// it easier to find similarly sized arrays in the codebase, as well as ensuring
/// they are consistently sized when specified in different places. Named
/// parameters also make it easier for readers to understand your code.
///
/// The values `0, 1, 2, 3, 4` are ignored by default.
///
/// TODO: Add user settings
///
/// ## Examples
/// Instead of:
/// ```f90
/// integer, dimension(10) :: x, y
/// ```
/// prefer:
/// ```f90
/// integer, parameter :: NUM_SPLINE_POINTS = 10
/// integer, dimension(NUM_SPLINE_POINTS) :: x, y
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MagicNumberInArraySize {
    value: i32,
}

impl Violation for MagicNumberInArraySize {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { value } = self;
        format!("Magic number in array size, consider replacing {value} with named `parameter`")
    }
}

const DEFAULT_ALLOWED_LITERALS: &[i32] = &[0, 1, 2, 3, 4];

impl AstRule for MagicNumberInArraySize {
    fn check(_settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
        // We're either looking for `type, dimension(X) :: variable` or `type :: variable(X)`
        let size = if node.kind() == "type_qualifier" {
            if node.child(0)?.to_text(source.source_text())?.to_lowercase() != "dimension" {
                return None;
            }
            node.child_with_name("argument_list")?
        } else {
            // sized_declarator
            node.child_with_name("size")?
        };

        let violations: Vec<_> = size
            .named_children(&mut size.walk())
            .filter_map(|child| match child.kind() {
                // Need to return a vec here to match next arm
                "number_literal" => Some(vec![child]),
                // This is `X:Y`, pull out the lower and upper bound separately
                "extent_specifier" => Some(
                    child
                        .named_children(&mut child.walk())
                        .filter(|extent| extent.kind() == "number_literal")
                        .collect_vec(),
                ),
                _ => None,
            })
            .flatten()
            .filter_map(|literal| {
                let value = literal
                    .to_text(source.source_text())?
                    .parse::<i32>()
                    .unwrap();
                if DEFAULT_ALLOWED_LITERALS.contains(&value) {
                    None
                } else {
                    Some((literal, value))
                }
            })
            .map(|(child, value)| Diagnostic::from_node(Self { value }, &child))
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["sized_declarator", "type_qualifier"]
    }
}

/// ## What it does
/// Checks for literal integers as units in IO statements.
///
/// ## Why is this bad?
/// Hardcoding unit numbers makes programs more brittle as it becomes harder to
/// verify units have been opened before reading/writing. Instead, units should
/// be passed in to procedures as arguments, or the `newunit=` argument used for
/// `open` statements. Having a named variable also makes it much clearer what a
/// given IO statement is for, and allows tools like LSP and IDEs to find all
/// references.
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
#[derive(ViolationMetadata)]
pub(crate) struct MagicIoUnit {
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
        let unit = literal_as_io_unit(node, src)?;

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
