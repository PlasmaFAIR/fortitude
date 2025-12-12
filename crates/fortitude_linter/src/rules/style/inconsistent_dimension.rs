use crate::FromAstNode;
use crate::ast::FortitudeNode;
use crate::ast::types::VariableDeclaration;
use crate::fix::edits::remove_variable_decl;
use crate::traits::{HasNode, TextRanged};
use anyhow::{Context, Result};
use itertools::Itertools;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for variable declarations that have both the `dimension` attribute
/// and an inline array specification.
///
/// ## Why is this bad?
/// Having both methods of declaring an array in one statement may be confusing
/// for the reader who may expect that all variables in the declaration have the
/// same shape as given by the `dimension` attribute. Prefer to declare
/// variables with different shapes to the `dimension` attribute on different
/// lines.
///
/// ## Example
/// ```f90
/// ! y and z are inconsistent with the `dimension` attribute
/// real, dimension(5) :: x, y(2), z(3, 4)
/// ```
///
/// Use instead:
/// ```f90
/// real, dimension(5) :: x
/// real :: y(2)
/// real :: z(3, 4)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct InconsistentArrayDeclaration;

impl AlwaysFixableViolation for InconsistentArrayDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Inconsistent specification of dimension".to_string()
    }

    fn fix_title(&self) -> String {
        "Move variable declaration to separate statement".to_string()
    }
}

fn fix_inconsistent_dimension(
    var: &Node,
    decl: &VariableDeclaration,
    src: &SourceFile,
) -> Result<Vec<Edit>> {
    let mut edits = vec![remove_variable_decl(var, decl, src)?];

    let type_ = decl.type_().as_str();
    let attrs = decl
        .attributes()
        .iter()
        .filter(|attr| !attr.kind().is_dimension())
        .filter_map(|attr| attr.node().to_text(src.source_text()))
        .join(", ");
    let first = if attrs.is_empty() {
        type_.to_string()
    } else {
        format!("{type_}, {attrs}")
    };
    let indent = decl.node().indentation(src);
    let line = format!(
        "\n{indent}{first} :: {}",
        var.to_text(src.source_text()).context("expected text")?
    );
    edits.push(Edit::insertion(line, decl.node().end_textsize()));

    Ok(edits)
}

pub fn check_inconsistent_dimension(
    decl_line: &VariableDeclaration,
    src: &SourceFile,
) -> Option<Vec<Diagnostic>> {
    if !decl_line
        .attributes()
        .iter()
        .any(|attr| attr.kind().is_dimension())
    {
        return None;
    }

    Some(
        decl_line
            .names()
            .iter()
            .filter(|name| {
                name.node().kind() == "sized_declarator"
                    || (name.node().kind() == "init_declarator"
                        && name.node().child(0).unwrap().kind() == "sized_declarator")
            })
            .filter_map(|node| {
                let mut edits = fix_inconsistent_dimension(node.node(), decl_line, src).ok()?;
                Some(
                    Diagnostic::from_node(InconsistentArrayDeclaration, node.node())
                        .with_fix(Fix::unsafe_edits(edits.remove(0), edits)),
                )
            })
            .collect_vec(),
    )
}
