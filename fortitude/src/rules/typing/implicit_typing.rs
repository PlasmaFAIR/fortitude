/// Defines rules that raise errors if implicit typing is in use.
use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

fn implicit_statement_is_none(node: &Node) -> bool {
    if let Some(child) = node.child(1) {
        return child.kind() == "none";
    }
    false
}

fn child_is_implicit_none(node: &Node) -> bool {
    if let Some(child) = node.child_with_name("implicit_statement") {
        return implicit_statement_is_none(&child);
    }
    false
}

/// ## What does it do?
/// Checks for missing `implicit none`
///
/// ## Why is this bad?
/// 'implicit none' should be used in all modules and programs, as implicit typing
/// reduces the readability of code and increases the chances of typing errors.
#[violation]
pub struct ImplicitTyping {
    entity: String,
}

impl Violation for ImplicitTyping {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity } = self;
        format!("{entity} missing 'implicit none'")
    }
}
impl AstRule for ImplicitTyping {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if !child_is_implicit_none(node) {
            let entity = node.kind().to_string();
            let block_stmt = node.child(0)?;
            return some_vec![Diagnostic::from_node(Self { entity }, &block_stmt)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module", "submodule", "program"]
    }
}

/// ## What it does
/// Checks for missing `implicit none` in interfaces
///
/// ## Why is this bad?
/// Interface functions and subroutines require 'implicit none', even if they are
/// inside a module that uses 'implicit none'.
#[violation]
pub struct InterfaceImplicitTyping {
    name: String,
}

impl Violation for InterfaceImplicitTyping {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("interface '{name}' missing 'implicit none'")
    }
}

impl AstRule for InterfaceImplicitTyping {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let parent = node.parent()?;
        if parent.kind() == "interface" && !child_is_implicit_none(node) {
            let name = node.kind().to_string();
            let interface_stmt = node.child(0)?;
            return some_vec![Diagnostic::from_node(Self { name }, &interface_stmt)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}

/// ## What it does
/// Checks for unnecessary `implicit none` in module procedures
///
/// ## Why is this bad?
/// If a module has 'implicit none' set, it is not necessary to set it in contained
/// functions and subroutines (except when using interfaces).
#[violation]
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
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if !implicit_statement_is_none(node) {
            return None;
        }
        let parent = node.parent()?;
        if matches!(parent.kind(), "function" | "subroutine") {
            for ancestor in parent.ancestors() {
                let kind = ancestor.kind();
                match kind {
                    "module" | "submodule" | "program" | "function" | "subroutine" => {
                        if !child_is_implicit_none(&ancestor) {
                            continue;
                        }
                        let entity = kind.to_string();
                        let fix = Fix::safe_edit(node.edit_delete(src));
                        return some_vec![
                            Diagnostic::from_node(Self { entity }, node).with_fix(fix)
                        ];
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

/// ## What it does
/// Checks if `implicit none` is missing `external`
///
/// ## Why is this bad?
/// `implicit none` disables implicit types of variables but still allows
/// implicit interfaces for procedures. Fortran 2018 added the ability to also
/// forbid implicit interfaces through `implicit none (external)`, enabling the
/// compiler to check the number and type of arguments and return values.
///
/// `implicit none` is equivalent to `implicit none (type)`, so the full
/// statement should be `implicit none (type, external)`.
#[violation]
pub struct ImplicitExternalProcedures {}

impl Violation for ImplicitExternalProcedures {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("'implicit none' missing 'external'")
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add `(external)` to 'implicit none'".to_string())
    }
}

impl AstRule for ImplicitExternalProcedures {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if !implicit_statement_is_none(node) {
            return None;
        }

        let text = node.to_text(src.source_text())?.to_lowercase();

        if !text.contains("external") {
            let edit = if let Some(type_node) = node
                .children(&mut node.walk())
                .find(|child| child.to_text(src.source_text()).unwrap().to_lowercase() == "type")
            {
                // Seems unlikely someone would have `implicit none (type)`
                // without `external` -- is that a sign they _explicitly_ don't
                // want it? That's probably still unwise though
                Edit::insertion(
                    ", external".to_string(),
                    TextSize::try_from(type_node.end_byte()).unwrap(),
                )
            } else {
                Edit::insertion(
                    " (type, external)".to_string(),
                    TextSize::try_from(node.end_byte()).unwrap(),
                )
            };
            let fix = Fix::unsafe_edit(edit);

            some_vec!(Diagnostic::from_node(Self {}, node).with_fix(fix))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["implicit_statement"]
    }
}
