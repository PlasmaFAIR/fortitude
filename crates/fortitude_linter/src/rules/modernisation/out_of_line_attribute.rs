use crate::ast::FortitudeNode;
use crate::locator::Locator;
use crate::symbol_table::{ParameterStatement, SymbolTables, get_name_node_of_declarator};
use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Violation};
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

impl Violation for OutOfLineAttribute {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            variable,
            attribute,
            ..
        } = self;
        format!("Out of line '{attribute}' attribute for variable '{variable}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self {
            variable,
            attribute,
            line,
            ..
        } = self;
        Some(format!(
            "Add '{attribute}' to '{variable}' declaration on line {line}"
        ))
    }
}

pub fn check_out_of_line_attribute(
    node: &Node,
    src: &SourceFile,
    symbol_table: &SymbolTables,
    locator: &Locator,
) -> Option<Vec<Diagnostic>> {
    let line_index = locator.to_index();

    if node.kind() == "parameter_statement" {
        let diagnostics = node
            .named_children(&mut node.walk())
            .filter_map(|parameter| {
                ParameterStatement::try_from_node(parameter, src.source_text()).ok()
            })
            .filter_map(|parameter| match symbol_table.get(&parameter.name) {
                Some(decl) => Some(Diagnostic::new(
                    OutOfLineAttribute {
                        variable: parameter.name,
                        attribute: "parameter".to_string(),
                        decl_location: decl.textrange(),
                        line: line_index.line_index(decl.node().start_textsize()),
                    },
                    parameter.location,
                )),
                None => None,
            })
            .collect_vec();
        return Some(diagnostics);
    }

    let variables = node
        .children_by_field_name("declarator", &mut node.walk())
        .map(|decl| {
            (
                decl,
                get_name_node_of_declarator(&decl)
                    .to_text(src.source_text())
                    .unwrap_or_default()
                    .to_string(),
            )
        })
        .collect_vec();

    Some(
        node.named_children(&mut node.walk())
            .filter(|child| child.kind() == "type_qualifier")
            .flat_map(|attribute| {
                let attribute_str = attribute
                    .to_text(src.source_text())
                    .unwrap_or_default()
                    .to_string();

                if matches!(
                    attribute_str.to_ascii_lowercase().as_ref(),
                    "external" | "intrinsic"
                ) {
                    return vec![];
                }

                variables
                    .iter()
                    .filter_map(|(var, name)| {
                        symbol_table.get(name).map(|decl| {
                            Diagnostic::new(
                                OutOfLineAttribute {
                                    variable: name.to_owned(),
                                    attribute: attribute_str.clone(),
                                    decl_location: decl.textrange(),
                                    line: line_index.line_index(decl.node().start_textsize()),
                                },
                                var.textrange(),
                            )
                        })
                    })
                    .collect_vec()
            })
            .collect_vec(),
    )
}
