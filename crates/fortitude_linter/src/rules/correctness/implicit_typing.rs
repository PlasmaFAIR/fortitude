/// Defines rules that raise errors if implicit typing is in use.
use crate::ast::{FortitudeNode, types::ImplicitType};
use crate::settings::{CheckSettings, FortranStandard};
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

fn insert_implicit_none(node: &Node, src: &SourceFile) -> Option<Fix> {
    // Find suitable place to insert `implicit none`, the line
    // after the last `use` statement, if any
    let last_use_statement_range = node
        .named_children(&mut node.walk())
        .filter_map(|child| {
            if child.kind() == "use_statement" {
                Some(child.textrange())
            } else {
                None
            }
        })
        .last()
        .or(Some(node.child(0)?.textrange()))?;

    // Get the start and end of the line
    let source_code = src.to_source_code();
    let source_location = source_code.source_location(last_use_statement_range.start());
    let line_start = source_code.line_start(source_location.row);
    let line_end = source_code.line_end(source_location.row);

    // TODO(peter): determine indentation of file using `Stylist` struct
    let indent = (last_use_statement_range.start() - line_start).to_usize();
    let insert = format!("{:indent$}implicit none\n", "");

    let edit = Edit::insertion(insert, line_end);

    Some(Fix::unsafe_edit(edit))
}

/// ## What does it do?
/// Checks for missing `implicit none`.
///
/// ## Why is this bad?
/// Very early Fortran determined the type of variables implicitly
/// from the first character of their name which saved lines in the
/// days of punchcards, and for backwards compatibility this is still
/// the default behaviour. However, the major downside is that typos
/// can silently introduce undefined variables and lead to hard to
/// track down bugs. For example:
///
/// ```f90
/// do i = 1, 10
///     print*, in
/// end do
/// ```
///
/// will print garbage.
///
/// 'implicit none' should be used in all modules and programs, as
/// implicit typing reduces the readability of code and increases the
/// chances of typing errors. Because it applies to all children of an
/// entity (all procedures in a module, for example), it _isn't_
/// required in every procedure, just the parent module or program if
/// there is one.
#[derive(ViolationMetadata)]
pub(crate) struct ImplicitTyping {
    entity: String,
}

impl Violation for ImplicitTyping {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity } = self;
        format!("{entity} missing 'implicit none'")
    }

    fn fix_title(&self) -> Option<String> {
        Some("Insert `implicit none`".to_string())
    }
}
impl AstRule for ImplicitTyping {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // If a procedure _isn't_ in a parent entity, then it should
        // have `implicit none`
        if matches!(node.kind(), "function" | "subroutine")
            && node.parent()?.kind() != "translation_unit"
        {
            return None;
        }

        let implicit_type = ImplicitType::from_scope(node, src)?;

        if implicit_type != ImplicitType::Missing {
            return None;
        }
        let entity = node.kind().to_string();
        let block_stmt = node.child(0)?;

        some_vec![
            Diagnostic::from_node(Self { entity }, &block_stmt)
                .with_fix(insert_implicit_none(node, src)?)
        ]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module", "submodule", "program", "subroutine", "function"]
    }
}

/// ## What it does
/// Checks for missing `implicit none` in interfaces
///
/// ## Why is this bad?
/// Interface functions and subroutines require 'implicit none', even if they are
/// inside a module that uses 'implicit none'.
#[derive(ViolationMetadata)]
pub(crate) struct InterfaceImplicitTyping {
    name: String,
}

impl Violation for InterfaceImplicitTyping {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("interface '{name}' missing 'implicit none'")
    }

    fn fix_title(&self) -> Option<String> {
        Some("Insert `implicit none`".to_string())
    }
}

impl AstRule for InterfaceImplicitTyping {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let parent = node.parent()?;
        if parent.kind() != "interface" {
            return None;
        }
        let implicit_type = ImplicitType::from_scope(node, src)?;
        if implicit_type == ImplicitType::Missing {
            let name = node.kind().to_string();
            let interface_stmt = node.child(0)?;
            return some_vec![
                Diagnostic::from_node(Self { name }, &interface_stmt)
                    .with_fix(insert_implicit_none(node, src)?)
            ];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["function", "subroutine"]
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
///
/// This rule is only active when targeting Fortran 2018 or later.
#[derive(ViolationMetadata)]
pub(crate) struct ImplicitExternalProcedures {}

impl Violation for ImplicitExternalProcedures {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'implicit none' missing 'external'".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add `(external)` to 'implicit none'".to_string())
    }
}

impl AstRule for ImplicitExternalProcedures {
    fn check(
        settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // implicit none (type, external) was added in Fortran 2018, so don't
        // run this rule if we're targeting an older standard.
        if settings.target_std < FortranStandard::F2018 {
            return None;
        }

        let edit = match ImplicitType::from_implicit_statement(node, src)? {
            ImplicitType::Missing
            | ImplicitType::Implicit
            | ImplicitType::NoneTypeExternal
            | ImplicitType::NoneExternal => {
                // If it's not `implicit none`, then we don't care about it.
                // If it's `implicit none (type, external)`, then it's already correct.
                // If it's `implicit none (external)`, then it's technically
                // correct, but probably a bad idea. C001/implicit-typing will
                // catch this.
                return None;
            }
            ImplicitType::None => {
                // If it's `implicit none` without `(external)`, then we want to fix it
                Edit::insertion(" (type, external)".to_string(), node.end_textsize())
            }
            ImplicitType::NoneType => {
                // If it's `implicit none (type)`, then we want to fix it.
                node.children(&mut node.walk())
                    .find(|child| {
                        child.to_text(src.source_text()).unwrap().to_lowercase() == "type"
                    })
                    .map(|type_node| {
                        Edit::insertion(", external".to_string(), type_node.end_textsize())
                    })?
            }
        };

        some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::unsafe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["implicit_statement"]
    }
}
