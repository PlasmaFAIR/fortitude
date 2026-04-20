/// Defines rules that raise errors if implicit typing is in use.
use crate::ast::{FortitudeNode, types::ImplicitStatement};
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for unnecessary `implicit none` in module procedures
///
/// ## Why is this bad?
/// If a module has 'implicit none' set, it is not necessary to set it in contained
/// functions and subroutines (except when using interfaces).
#[derive(ViolationMetadata)]
pub struct SuperfluousImplicitNone {
    entity: String,
}

impl AlwaysFixableViolation for SuperfluousImplicitNone {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity } = self;
        format!("'implicit none' set on the enclosing {entity}")
    }

    fn fix_title(&self) -> String {
        "Remove unnecessary 'implicit none'".to_string()
    }
}

impl AstRule for SuperfluousImplicitNone {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let stmt = ImplicitStatement::try_from_node(*node, src)?;
        // If this isn't an `implicit none` statement, then we don't care about it.
        if stmt.is_not_implicit_none() {
            return None;
        }
        let parent = node.parent()?;
        if matches!(parent.kind(), "function" | "subroutine") {
            for ancestor in parent.ancestors() {
                let kind = ancestor.kind();
                match kind {
                    "interface" => {
                        // Implicit none doesn't propagate through interfaces.
                        break;
                    }
                    "module" | "submodule" | "program" | "function" | "subroutine" => {
                        match ImplicitStatement::try_from_scope(&ancestor, src) {
                            None => {
                                // If the ancestor doesn't have any `implicit` statement, then
                                // keep searching up the tree for a higher-level entity with
                                // `implicit none`, if any. If we reach the top without finding
                                // one, then it's not a problem.
                                continue;
                            }
                            Some(ancestor_stmt) => {
                                // If the ancestor statement is equivalent to this one,
                                // then this one is superfluous and should be removed.
                                if stmt.is_equivalent_to(&ancestor_stmt) {
                                    let entity = kind.to_string();
                                    let fix = Fix::safe_edit(node.edit_delete(src));
                                    return some_vec![
                                        Diagnostic::from_node(Self { entity }, node).with_fix(fix)
                                    ];
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["implicit_statement"]
    }
}
