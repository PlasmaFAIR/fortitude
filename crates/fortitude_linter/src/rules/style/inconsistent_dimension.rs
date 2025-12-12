use crate::FromAstNode;
use crate::Rule;
use crate::ast::FortitudeNode;
use crate::ast::types::VariableDeclaration;
use crate::fix::edits::remove_variable_decl;
use crate::rule_table::RuleTable;
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

fn check_inconsistent_dimension(
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
            .filter(|name| name.size().is_some())
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

/// ## What it does
/// Checks for variable declarations that mix declarations of both scalars and
/// arrays.
///
/// ## Why is this bad?
/// Mixing declarations of scalars and arrays in one statement may mislead the
/// reader into thinking all variables are scalar. Prefer to declare arrays in
/// separate statements to scalars.
///
/// ## Example
/// ```f90
/// ! only y is an array here
/// real :: x, y(2), z
/// ```
///
/// Use instead:
/// ```f90
/// real :: x, z
/// real :: y(2)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MixedScalarArrayDeclaration;

impl AlwaysFixableViolation for MixedScalarArrayDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Mixed declaration of scalar(s) and array".to_string()
    }

    fn fix_title(&self) -> String {
        "Move variable declaration to separate statement".to_string()
    }
}

fn check_mixed_scalar_array(
    decl_line: &VariableDeclaration,
    src: &SourceFile,
) -> Option<Vec<Diagnostic>> {
    if decl_line
        .attributes()
        .iter()
        .any(|attr| attr.kind().is_dimension())
    {
        return None;
    }

    // Don't complain if there aren't any scalars
    if decl_line.names().iter().all(|name| name.size().is_some()) {
        return None;
    }

    Some(
        decl_line
            .names()
            .iter()
            .filter(|name| name.size().is_some())
            .filter_map(|node| {
                let mut edits = fix_inconsistent_dimension(node.node(), decl_line, src).ok()?;
                Some(
                    Diagnostic::from_node(MixedScalarArrayDeclaration, node.node())
                        .with_fix(Fix::unsafe_edits(edits.remove(0), edits)),
                )
            })
            .collect_vec(),
    )
}

pub fn check_inconsistent_dimension_rules(
    rules: &RuleTable,
    decl_line: &VariableDeclaration,
    src: &SourceFile,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();

    if rules.enabled(Rule::InconsistentArrayDeclaration) {
        if let Some(violation) = check_inconsistent_dimension(decl_line, src) {
            violations.extend(violation);
        }
    }
    if rules.enabled(Rule::MixedScalarArrayDeclaration) {
        if let Some(violation) = check_mixed_scalar_array(decl_line, src) {
            violations.extend(violation);
        }
    }

    violations
}
