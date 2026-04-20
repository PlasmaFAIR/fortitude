/// Defines rules that raise errors if implicit typing is in use.
use crate::ast::{FortitudeNode, types::ImplicitStatement};
use crate::settings::{CheckSettings, FortranStandard};
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Inserts `implicit none` in the current scope. Should be called on a program,
/// module, submodule, function, or subroutine.
fn insert_implicit_none(node: &Node, src: &SourceFile) -> Option<Edit> {
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
    Some(Edit::insertion(insert, line_end))
}

/// Replaces an existing implicit statement with `implicit none`. Used when
/// there is an implicit statement such as `implicit real(a-z)`.
fn replace_with_implicit_none(node: &Node, src: &SourceFile) -> Edit {
    node.edit_replacement(src, "implicit none".to_owned())
}

/// Assuming we have `implicit none (external)` for some cursed reason, adds
/// `type` to it to make it `implicit none (type, external)`.
fn add_type_to_implicit_none(node: &Node, src: &SourceFile) -> Option<Edit> {
    node.children(&mut node.walk())
        .find(|child| child.to_text(src.source_text()).unwrap().to_lowercase() == "external")
        .map(|external_node| Edit::insertion("type, ".to_string(), external_node.start_textsize()))
}

enum ImplicitTypingErrorType {
    NoImplicitStatement,
    NotImplicitNone,
    ExternalWithoutType,
}

impl ImplicitTypingErrorType {
    fn fix_title(&self) -> String {
        match self {
            Self::NoImplicitStatement => "Insert `implicit none`".to_string(),
            Self::NotImplicitNone => "Change to `implicit none`".to_string(),
            Self::ExternalWithoutType => "Change to `implicit none (type, external)`".to_string(),
        }
    }
}

struct ImplicitTypingEdit {
    edit: Edit,
    error_type: ImplicitTypingErrorType,
}

impl ImplicitTypingEdit {
    /// Called on the scope that should contain an `implicit none` statement.
    /// Returns an edit if a violation is found, otherwise returns `None`.
    fn try_from_scope(node: &Node, src: &SourceFile) -> Option<Self> {
        match ImplicitStatement::try_from_scope(node, src) {
            Some(stmt) => {
                if stmt.is_implicit_none_type() {
                    // This is sufficient for these rules.
                    // implicit-external-procedures will handle missing `external` in `implicit none (type)`.
                    return None;
                }
                if stmt.is_implicit_none_external() {
                    // User has specified `implicit none (external)`, which is
                    // technically correct but probably a mistake, so we want to
                    // fix it to `implicit none (type, external)`.
                    let error_type = ImplicitTypingErrorType::ExternalWithoutType;
                    let edit = add_type_to_implicit_none(stmt.node(), src)?;
                    return Some(Self { edit, error_type });
                }
                // If we get here, then there is an implicit statement but it's
                // not `implicit none`. Should replace the whole statement with
                // `implicit none`.
                let error_type = ImplicitTypingErrorType::NotImplicitNone;
                let edit = replace_with_implicit_none(stmt.node(), src);
                Some(Self { edit, error_type })
            }
            None => {
                // Missing implicit statement -- should insert one.
                let error_type = ImplicitTypingErrorType::NoImplicitStatement;
                let edit = insert_implicit_none(node, src)?;
                Some(Self { edit, error_type })
            }
        }
    }
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
    error_type: ImplicitTypingErrorType,
}

impl Violation for ImplicitTyping {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { entity, .. } = self;
        format!("{entity} uses implicit typing")
    }

    fn fix_title(&self) -> Option<String> {
        Some(self.error_type.fix_title())
    }
}

impl AstRule for ImplicitTyping {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // Run on functions and subroutines only if they aren't in a module,
        // program, or submodule. This rule will catch implicit typing in the
        // parent enttity, so we don't need to check it in the children.
        if matches!(node.kind(), "function" | "subroutine")
            && node.parent()?.kind() != "translation_unit"
        {
            return None;
        }

        let ImplicitTypingEdit { edit, error_type } =
            ImplicitTypingEdit::try_from_scope(node, src)?;
        let entity = node.kind().to_string();
        let block_stmt = node.child(0)?;

        some_vec![
            Diagnostic::from_node(Self { entity, error_type }, &block_stmt)
                .with_fix(Fix::unsafe_edit(edit))
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
    error_type: ImplicitTypingErrorType,
}

impl Violation for InterfaceImplicitTyping {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, .. } = self;
        format!("interface '{name}' uses implicit typing")
    }

    fn fix_title(&self) -> Option<String> {
        Some(self.error_type.fix_title())
    }
}

impl AstRule for InterfaceImplicitTyping {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // Exit early if we're not in an interface.
        let parent = node.parent()?;
        if parent.kind() != "interface" {
            return None;
        }

        let ImplicitTypingEdit { edit, error_type } =
            ImplicitTypingEdit::try_from_scope(node, src)?;
        let name = node.kind().to_string();
        let interface_stmt = node.child(0)?;

        some_vec![
            Diagnostic::from_node(Self { name, error_type }, &interface_stmt)
                .with_fix(Fix::unsafe_edit(edit))
        ]
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

        let stmt = ImplicitStatement::try_from_node(*node, src)?;

        if stmt.is_implicit_none_external() {
            // If `external` is already present, then it's correct.
            return None;
        }

        if stmt.is_not_implicit_none() {
            // This isn't `implicit none` at all, so we don't care about it.
            return None;
        }

        // If we get here, it's either `implicit none` or `implicit none
        // (type)`, so we want to add `external` to it.
        let edit = node
            .children(&mut node.walk())
            .find(|child| child.to_text(src.source_text()).unwrap().to_lowercase() == "type")
            .map_or_else(
                || Edit::insertion(" (type, external)".to_string(), node.end_textsize()),
                |type_node| Edit::insertion(", external".to_string(), type_node.end_textsize()),
            );
        some_vec!(Diagnostic::from_node(Self {}, node).with_fix(Fix::unsafe_edit(edit)))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["implicit_statement"]
    }
}
