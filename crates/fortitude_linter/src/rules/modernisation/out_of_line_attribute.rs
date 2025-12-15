use crate::ast::FortitudeNode;
use crate::ast::types::{ParameterStatement, Variable, get_name_node_of_declarator};
use crate::fix::edits::{
    add_attribute_to_var_decl, remove_from_comma_sep_stmt, remove_variable_decl,
};
use crate::traits::{HasNode, TextRanged};
use crate::{AstRule, FromAstNode, SymbolTables};

use anyhow::{Context, Result};
use itertools::Itertools;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::{OneIndexed, SourceFile};
use ruff_text_size::TextRange;
use tree_sitter::Node;

/// ## What it does
/// Checks for variable attributes which are specified separately to the
/// variable declaration.
///
/// ## Why is this bad?
/// Using separate attribute specification statements (or "out-of-line
/// attributes") makes the code harder to read by splitting up the important
/// information about a variable. Instead, give attributes in-line with the
/// variable declaration. This way, readers only need to look in one place.
///
/// ## Example
/// ```f90
/// integer :: nx
/// real :: x_grid
/// parameter (nx = 42)
/// dimension x_grid(nx)
/// ```
///
/// Use instead:
/// ```f90
/// integer, parameter :: nx = 42
/// real, dimension(nx) :: x_grid
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct OutOfLineAttribute {
    variable: String,
    attribute: String,
    #[allow(dead_code)]
    decl_location: TextRange,
    line: OneIndexed,
}

impl AlwaysFixableViolation for OutOfLineAttribute {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            variable,
            attribute,
            ..
        } = self;
        format!("Out of line '{attribute}' attribute for variable '{variable}'")
    }

    fn fix_title(&self) -> String {
        let Self {
            variable,
            attribute,
            line,
            ..
        } = self;
        format!("Add '{attribute}' to '{variable}' declaration on line {line}")
    }
}

fn remove_from_parameter_stmt(var: &Node, stmt: &Node, src: &SourceFile) -> Result<Edit> {
    let params = stmt
        .named_children(&mut stmt.walk())
        .filter(|child| child.kind() == "parameter_assignment")
        .collect_vec();

    remove_from_comma_sep_stmt(var, stmt, &params, src)
}

fn remove_from_attribute_stmt(var: &Node, stmt: &Node, src: &SourceFile) -> Result<Edit> {
    let params = stmt
        .children_by_field_name("declarator", &mut stmt.walk())
        .collect_vec();

    remove_from_comma_sep_stmt(var, stmt, &params, src)
}

fn fix_out_of_line_parameter(
    attr_node: &Node,
    var_in_attr: &ParameterStatement,
    var: &Variable,
    src: &SourceFile,
) -> Result<Vec<Edit>> {
    // Remove from variable_modification
    let mut edits = vec![remove_from_parameter_stmt(
        &var_in_attr.node,
        attr_node,
        src,
    )?];

    let extra = format!(" = {}", var_in_attr.expression);

    edits.extend(make_fix(var, "parameter", extra, src)?);
    Ok(edits)
}

fn fix_out_of_line_attribute(
    attr_node: &Node,
    var_in_attr: &Node,
    var: &Variable,
    src: &SourceFile,
) -> Result<Vec<Edit>> {
    // Remove from variable_modification
    let mut edits = vec![remove_from_attribute_stmt(var_in_attr, attr_node, src)?];

    let attr = attr_node
        .child(0)
        .context("missing child 0")?
        .to_text(src.source_text())
        .context("missing text")?;

    // `dimension` and `allocatable` both need the size node, but they'll put
    // them in different places:
    // - `dimension(size) :: var
    // - `allocatable :: var(size)`
    let size = if let Some(size) = var_in_attr.child_with_name("size") {
        Some(size.to_text(src.source_text()).context("missing text")?)
    } else {
        None
    };

    let new_attr = if attr.eq_ignore_ascii_case("dimension") {
        let size = size.context("expected 'size' for 'dimension'")?;
        format!("{attr}{size}")
    } else {
        attr.to_string()
    };

    let extra = if attr.eq_ignore_ascii_case("allocatable")
        && let Some(size) = size
    {
        size
    } else {
        ""
    }
    .to_string();

    edits.extend(make_fix(var, &new_attr, extra, src)?);
    Ok(edits)
}

/// Move a variable attribute from a standalone statement to the variable
/// declaration, removing any empty statements, and making sure we only apply
/// the attribute to one variable, and not any others declared in the same
/// statement
fn make_fix(var: &Variable, new_attr: &str, extra: String, src: &SourceFile) -> Result<Vec<Edit>> {
    let mut edits = Vec::new();

    let decl = var.decl_statement();
    if decl.names().len() == 1 {
        // If only variable in decl statement:
        //   -> add attribute to decl statement
        edits.push(add_attribute_to_var_decl(decl, new_attr));
        if !extra.is_empty() {
            edits.push(Edit::insertion(extra, var.node().end_textsize()));
        }
    } else {
        // Otherwise:
        //   -> remove variable from decl statement
        edits.push(remove_variable_decl(var.node(), decl, src)?);
        //   -> add new decl statement with attribute
        let type_ = decl.type_().as_str();
        let attrs = decl
            .attributes()
            .iter()
            .filter_map(|attr| attr.node().to_text(src.source_text()))
            .join(", ");
        let first = if attrs.is_empty() {
            type_.to_string()
        } else {
            format!("{type_}, {attrs}")
        };
        let indent = decl.node().indentation(src);
        let line = format!("\n{indent}{first}, {new_attr} :: {}{extra}", var.name(),);
        edits.push(Edit::insertion(line, decl.node().end_textsize()));
    }

    Ok(edits)
}

impl AstRule for OutOfLineAttribute {
    fn check(
        _settings: &crate::settings::CheckSettings,
        node: &Node,
        src: &SourceFile,
        symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let code = src.to_source_code();

        if node.kind() == "parameter_statement" {
            // Parameter statements have slightly different syntax, and have
            // different nodes in the AST, so handle separately
            let diagnostics = node
                .named_children(&mut node.walk())
                .filter_map(|parameter| {
                    ParameterStatement::try_from_node(parameter, src.source_text()).ok()
                })
                .filter_map(|parameter| match symbol_table.get(&parameter.name) {
                    Some(var) => {
                        let mut edit =
                            fix_out_of_line_parameter(node, &parameter, &var, src).unwrap();
                        Some(
                            Diagnostic::from_node(
                                OutOfLineAttribute {
                                    variable: parameter.name,
                                    attribute: "parameter".to_string(),
                                    decl_location: var.textrange(),
                                    line: code.line_index(var.node().start_textsize()),
                                },
                                &parameter.node,
                            )
                            .with_fix(Fix::unsafe_edits(edit.remove(0), edit)),
                        )
                    }
                    None => None,
                })
                .collect_vec();
            return Some(diagnostics);
        }

        let attribute_str = node
            .child_with_name("type_qualifier")?
            .to_text(src.source_text())?;

        if attribute_str.eq_ignore_ascii_case("external")
            || attribute_str.eq_ignore_ascii_case("intrinsic")
        {
            // These don't have variable declarations to apply to?
            return None;
        }

        Some(
            node.children_by_field_name("declarator", &mut node.walk())
                .map(|decl| {
                    (
                        decl,
                        get_name_node_of_declarator(&decl)
                            .to_text(src.source_text())
                            .unwrap_or_default()
                            .to_string(),
                    )
                })
                .filter_map(|(decl, name)| {
                    symbol_table.get(&name).map(|var| {
                        let mut edit = fix_out_of_line_attribute(node, &decl, &var, src).unwrap();

                        Diagnostic::new(
                            OutOfLineAttribute {
                                variable: name,
                                attribute: attribute_str.to_string(),
                                decl_location: decl.textrange(),
                                line: code.line_index(var.node().start_textsize()),
                            },
                            decl.textrange(),
                        )
                        .with_fix(Fix::unsafe_edits(edit.remove(0), edit))
                    })
                })
                .collect_vec(),
        )
    }
    fn entrypoints() -> Vec<&'static str> {
        vec!["parameter_statement", "variable_modification"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::TextSize;
    use tree_sitter::Parser;

    #[test]
    fn test_remove_from_parameter_stmt() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  parameter(x = 1)
  parameter(a = 1, b=2,&
    c=3)
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;
        let test_source = SourceFileBuilder::new("test.f90", code).finish();

        let parameter_stmt_0 = root.child(1).unwrap();
        let x = parameter_stmt_0.named_child(0).unwrap();

        let parameter_stmt_1 = root.child(2).unwrap();
        let a = parameter_stmt_1.named_child(0).unwrap();
        let b = parameter_stmt_1.named_child(1).unwrap();
        let c = parameter_stmt_1.named_child(2).unwrap();

        let remove_x = remove_from_parameter_stmt(&x, &parameter_stmt_0, &test_source)?;
        assert_eq!(
            remove_x,
            Edit::deletion(TextSize::new(13), TextSize::new(32))
        );

        let remove_a = remove_from_parameter_stmt(&a, &parameter_stmt_1, &test_source)?;
        assert_eq!(
            remove_a,
            Edit::deletion(TextSize::new(44), TextSize::new(50))
        );

        let remove_b = remove_from_parameter_stmt(&b, &parameter_stmt_1, &test_source)?;
        assert_eq!(
            remove_b,
            Edit::deletion(TextSize::new(51), TextSize::new(55))
        );

        let remove_c = remove_from_parameter_stmt(&c, &parameter_stmt_1, &test_source)?;
        assert_eq!(
            remove_c,
            Edit::deletion(TextSize::new(54), TextSize::new(64))
        );

        Ok(())
    }

    #[test]
    fn test_remove_from_dimension_stmt() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  dimension x(2)
  DIMENSION A(*), B(*) &
      ,C(*)
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;
        let test_source = SourceFileBuilder::new("test.f90", code).finish();

        let dimension_stmt_0 = root.child(1).unwrap();
        let x = dimension_stmt_0.named_child(1).unwrap();

        let dimension_stmt_1 = root.child(2).unwrap();
        let a = dimension_stmt_1.named_child(1).unwrap();
        let b = dimension_stmt_1.named_child(2).unwrap();
        let c = dimension_stmt_1.named_child(3).unwrap();

        let remove_x = remove_from_attribute_stmt(&x, &dimension_stmt_0, &test_source)?;
        assert_eq!(
            remove_x,
            Edit::deletion(TextSize::new(13), TextSize::new(30))
        );

        let remove_a = remove_from_attribute_stmt(&a, &dimension_stmt_1, &test_source)?;
        assert_eq!(
            remove_a,
            Edit::deletion(TextSize::new(42), TextSize::new(47))
        );

        let remove_b = remove_from_attribute_stmt(&b, &dimension_stmt_1, &test_source)?;
        assert_eq!(
            remove_b,
            Edit::deletion(TextSize::new(48), TextSize::new(62))
        );

        let remove_c = remove_from_attribute_stmt(&c, &dimension_stmt_1, &test_source)?;
        assert_eq!(
            remove_c,
            Edit::deletion(TextSize::new(52), TextSize::new(66))
        );

        Ok(())
    }
}
