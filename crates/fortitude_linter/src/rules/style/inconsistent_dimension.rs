use crate::FromAstNode;
use crate::Rule;
use crate::ast::FortitudeNode;
use crate::ast::types::NameDecl;
use crate::ast::types::VariableDeclaration;
use crate::fix::edits::remove_variable_decl;
use crate::settings::CheckSettings;
use crate::traits::{HasNode, TextRanged};
use anyhow::{Context, Result};
use itertools::Itertools;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use settings::PreferAttribute;

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
/// ## Automatic Fix
/// The automatic fix for this moves the variable declaration to a new
/// statement, and is unsafe as it may clobber comments.
///
/// You can use `check.inconsistent-dimension.prefer-attribute` to control
/// whether to put a `dimension` attribute on the new declaration or not.
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
///
/// ## Options
/// - `check.inconsistent-dimensions.prefer-attribute`
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

fn dimension_attribute_and_shape(
    var: &NameDecl,
    decl: &VariableDeclaration,
    src: &SourceFile,
    add_attribute: bool,
) -> Result<(String, String)> {
    // Get the shape, if declared on the variable name
    let size = var
        .size()
        .map(|s| s.to_text(src.source_text()).context("expected text"))
        .transpose()?;

    if add_attribute {
        // Adding dimension attribute, removing decl size
        let size = size.context("expected size")?;
        Ok((format!(", dimension{size}"), var.name().to_string()))
    } else {
        // Removing dimension attribute, adding decl size

        let size = size
            .or_else(|| {
                let dim = decl
                    .attributes()
                    .iter()
                    .find(|attr| attr.kind().is_dimension());

                dim?.node()
                    .child_with_name("argument_list")?
                    .to_text(src.source_text())
            })
            .context("expected either size or dimension attribute")?;
        Ok(("".to_string(), format!("{}{size}", var.name())))
    }
}

fn fix_inconsistent_dimension(
    var: &NameDecl,
    decl: &VariableDeclaration,
    src: &SourceFile,
    prefer_attribute: PreferAttribute,
) -> Result<Vec<Edit>> {
    let mut edits = vec![remove_variable_decl(var.node(), decl, src)?];

    let (new_attr, var_str) =
        dimension_attribute_and_shape(var, decl, src, prefer_attribute.is_always())?;

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
    let init = if let Some(init) = var.init() {
        let init_value = init.to_text(src.source_text()).context("expected text")?;
        format!(" = {init_value}")
    } else {
        "".to_string()
    };

    let indent = decl.node().indentation(src);
    let line = format!("{indent}{first}{new_attr} :: {var_str}{init}\n");

    let source_code = src.to_source_code();
    let line_index = source_code.line_index(decl.node().end_textsize());
    let line_end = src.to_source_code().line_end(line_index);
    edits.push(Edit::insertion(line, line_end));

    Ok(edits)
}

fn check_inconsistent_dimension(
    decl_line: &VariableDeclaration,
    src: &SourceFile,
    prefer_attribute: PreferAttribute,
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
                let mut edits =
                    fix_inconsistent_dimension(node, decl_line, src, prefer_attribute).ok()?;
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
/// ## Automatic Fix
/// The automatic fix for this moves the variable declaration to a new
/// statement, and is unsafe as it may clobber comments.
///
/// You can use `check.inconsistent-dimension.prefer-attribute` to control
/// whether to put a `dimension` attribute on the new declaration or not.
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
///
/// ## Options
/// - `check.inconsistent-dimensions.prefer-attribute`
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
    prefer_attribute: PreferAttribute,
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
                // FIXME: use of `.ok()?` means error in fix skips the
                // diagnostic completely, hiding bugs.
                let mut edits =
                    fix_inconsistent_dimension(node, decl_line, src, prefer_attribute).ok()?;
                Some(
                    Diagnostic::from_node(MixedScalarArrayDeclaration, node.node())
                        .with_fix(Fix::unsafe_edits(edits.remove(0), edits)),
                )
            })
            .collect_vec(),
    )
}

/// ## What it does
/// Checks for variable array declarations that either do or do not use the
/// `dimension` attribute.
///
/// ## Why is this bad?
/// Array variables in Fortran can be declared using either the `dimension`
/// attribute, or with an "array-spec" (shape) in parentheses:
///
/// ```f90
/// ! With an attribute
/// integer, dimension(2) :: x
/// ! With a shape in brackets
/// integer :: x(2)
/// ```
///
/// The two forms are exactly equivalent, but some projects prefer to only use
/// form over the other for consistency.
///
/// !!! note
///     This rule can feel quite pedantic, and so as well as enabling it, you
///     must also set `check.inconsistent-dimensions.prefer-attribute` to either
///     `"always"` or `"never"` to require the `dimension` attribute or to
///     remove it, respectively. The default value of `"keep"` effectively turns
///     this rule off.
///
/// ## Options
/// - `check.inconsistent-dimensions.prefer-attribute`
#[derive(ViolationMetadata)]
pub(crate) struct BadArrayDeclaration {
    prefer_attribute: PreferAttribute,
}

impl AlwaysFixableViolation for BadArrayDeclaration {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Bad declaration of array".to_string()
    }

    fn fix_title(&self) -> String {
        if self.prefer_attribute.is_always() {
            "Add `dimension` attribute to declaration".to_string()
        } else {
            "Remove `dimension` attribute from declaration".to_string()
        }
    }
}

fn check_bad_array_decl(
    decl_line: &VariableDeclaration,
    src: &SourceFile,
    prefer_attribute: PreferAttribute,
) -> Option<Vec<Diagnostic>> {
    let has_dim = decl_line
        .attributes()
        .iter()
        .any(|attr| attr.kind().is_dimension());

    if has_dim && prefer_attribute.is_always() {
        // We've got a `dimension` attriute and want one
        return None;
    } else if !has_dim && prefer_attribute.is_never() {
        // We don't have a `dimension` attribute and don't want one
        return None;
    }

    Some(
        decl_line
            .names()
            .iter()
            .filter(|name| {
                (name.size().is_some() && prefer_attribute.is_always())
                    || (name.size().is_none() && prefer_attribute.is_never())
            })
            .map(|node| {
                // FIXME: use of `ok` hides bugs
                let mut edits = fix_inconsistent_dimension(node, decl_line, src, prefer_attribute)
                    .ok()
                    .unwrap();
                Diagnostic::from_node(BadArrayDeclaration { prefer_attribute }, node.node())
                    .with_fix(Fix::unsafe_edits(edits.remove(0), edits))
            })
            .collect_vec(),
    )
}

pub fn check_inconsistent_dimension_rules(
    settings: &CheckSettings,
    decl_line: &VariableDeclaration,
    src: &SourceFile,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();

    let rules = &settings.rules;
    let prefer_attribute = settings.inconsistent_dimension.prefer_attribute;

    if rules.enabled(Rule::InconsistentArrayDeclaration) {
        if let Some(violation) = check_inconsistent_dimension(decl_line, src, prefer_attribute) {
            violations.extend(violation);
        }
    }
    if rules.enabled(Rule::MixedScalarArrayDeclaration) {
        if let Some(violation) = check_mixed_scalar_array(decl_line, src, prefer_attribute) {
            violations.extend(violation);
        }
    }
    if rules.enabled(Rule::BadArrayDeclaration) {
        match prefer_attribute {
            // Do nothing!
            PreferAttribute::Keep => (),
            _ => {
                if let Some(violation) = check_bad_array_decl(decl_line, src, prefer_attribute) {
                    violations.extend(violation);
                }
            }
        }
    }
    violations
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;
    use strum_macros::EnumIs;

    #[derive(
        CacheKey,
        Clone,
        Copy,
        Debug,
        Default,
        Deserialize,
        EnumIs,
        Eq,
        Hash,
        PartialEq,
        Serialize,
        clap::ValueEnum,
        strum_macros::Display,
    )]
    #[serde(rename_all = "lowercase")]
    #[strum(serialize_all = "lowercase")]
    pub enum PreferAttribute {
        #[default]
        Keep,
        Always,
        Never,
    }

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub prefer_attribute: PreferAttribute,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.inconsistent_dimension",
                fields = [self.prefer_attribute]
            }
            Ok(())
        }
    }
}
