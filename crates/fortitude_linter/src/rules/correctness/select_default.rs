use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks that `select case` statements have a `case default`.
///
/// ## Why is this bad?
/// Select statements without a default case can lead to incomplete handling of
/// the possible options. If a value isn't handled by any of the cases, the
/// program will continue execution, which may lead to surprising results.
/// Unfortunately, because Fortran doesn't have proper enums, it's not possible
/// for the compiler to issue warnings for non-exhaustive cases. Having a default
/// case allows for the program to gracefully handle errors.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDefaultCase {}

impl Violation for MissingDefaultCase {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing default case may not handle all values".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'case default'".to_string())
    }
}

impl AstRule for MissingDefaultCase {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        _src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let has_default = node
            .named_children(&mut node.walk())
            .filter(|child| child.kind() == "case_statement")
            .any(|case| {
                case.named_children(&mut case.walk())
                    .any(|child| child.kind() == "default")
            });

        if has_default {
            None
        } else {
            some_vec!(Diagnostic::from_node(Self {}, node))
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["select_case_statement"]
    }
}
