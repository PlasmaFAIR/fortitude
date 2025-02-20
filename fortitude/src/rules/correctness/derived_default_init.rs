use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What it does
/// Checks for uninitialised pointer variables inside derived types
///
/// ## Why is this bad?
/// Pointers inside derived types are undefined by default, and their
/// status cannot be tested by intrinsics such as `associated`. Pointer
/// variables should be initialised by either associating them with another
/// variable, or associating to `null()`.
///
///
/// ## Examples
/// For example, this derived type:
///
/// ```f90
/// type mytype
///     real :: val1
///     integer :: val2
///
///     real, pointer :: pReal1
///
///     integer, pointer :: pInt1 => null()
///     integer, pointer :: pI1
///     integer, pointer :: pI2 => null(), pI3
/// end mytype
/// ```
/// will have the pointers `pReal1`, `pI1`, and `pI3` uninitialised
/// whenever it is created. Instead, they should be initialised like:
/// ```f90
/// type mytype
///     real :: val1
///     integer :: val2
///
///     real, pointer :: pReal1
///
///     integer, pointer :: pInt1 => null()
///     integer, pointer :: pI1 => null()
///     integer, pointer :: pI2 => null(), pI3  => null()
/// end mytype
/// ```
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Section 8.5.3/8.5.4.
/// - Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
///   University Press, Rule 136, p. 189.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDefaultPointerInitalisation {
    var: String,
}

impl Violation for MissingDefaultPointerInitalisation {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { var } = self;
        format!("pointer component '{var}' does not have a default initialiser")
    }

    fn fix_title(&self) -> Option<String> {
        Some("Associate to a known value, such as 'null()'".to_string())
    }
}

impl AstRule for MissingDefaultPointerInitalisation {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // Only operate on derived types
        if node.parent()?.kind() != "derived_type_definition" {
            return None;
        }

        let mut node_cursor = node.walk();

        // Only check pointer variables
        if !node.named_children(&mut node_cursor).any(|node| {
            node.kind() == "type_qualifier"
                && node
                    .to_text(src.source_text())
                    .unwrap_or("")
                    .to_lowercase()
                    .starts_with("pointer")
        }) {
            return None;
        }

        let violations: Vec<Diagnostic> = node
            .named_descendants()
            .filter(|node| node.kind() == "identifier")
            .filter(|node| match node.parent() {
                None => false,
                Some(parent) => parent.kind() == "variable_declaration",
            })
            .map(|node| {
                let var_name = node.to_text(src.source_text()).unwrap_or("").to_string();

                let init_var = format!(" => null()");
                let start_pos = TextSize::try_from(node.end_byte()).unwrap();
                let fix = Fix::unsafe_edit(Edit::insertion(init_var, start_pos));

                Diagnostic::from_node(Self { var: var_name }, &node).with_fix(fix)
            })
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["variable_declaration"]
    }
}
