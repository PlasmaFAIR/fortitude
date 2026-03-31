/// Defines rules that raise errors if implicit typing is in use.
use crate::ast::{FortitudeNode, types::ImplicitType};
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
        // If this isn't an `implicit none` statement, then we don't care about it.
        let implicit_type = ImplicitType::from_implicit_statement(node, src)?;
        if implicit_type == ImplicitType::Implicit {
            return None;
        }
        let parent = node.parent()?;
        if matches!(parent.kind(), "function" | "subroutine") {
            for ancestor in parent.ancestors() {
                let kind = ancestor.kind();
                match kind {
                    "module" | "submodule" | "program" | "function" | "subroutine" => {
                        match ImplicitType::from_scope(&ancestor, src)? {
                            ImplicitType::Missing => {
                                // Keep searching up the tree for a higher-level
                                // entity with `implicit none`, if any. If we
                                // reach the top without finding one, then it's
                                // not a problem.
                                continue;
                            }
                            ImplicitType::Implicit => {
                                // If we find an ancestor entity with `implicit
                                // type(a-z)`, then this one is not superfluous.
                                break;
                            }
                            ancestor_implicit_type => {
                                // If we find an ancestor entity with `implicit
                                // none`, then this one is superfluous provided
                                // it is equivalent to the ancestor's `implicit
                                // none` (e.g. `implicit none (type)` is not
                                // equivalent to `implicit none (external)`, but
                                // is to `implicit none`).
                                if !implicit_type.equivalent_to(&ancestor_implicit_type) {
                                    break;
                                }
                                let entity = kind.to_string();
                                let fix = Fix::safe_edit(node.edit_delete(src));
                                return some_vec![
                                    Diagnostic::from_node(Self { entity }, node).with_fix(fix)
                                ];
                            }
                        }
                    }
                    "interface" => {
                        break;
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
