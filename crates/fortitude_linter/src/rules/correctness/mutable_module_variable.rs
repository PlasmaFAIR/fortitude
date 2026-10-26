use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use fortitude_sitter::Node;
use fortitude_sitter::ast::types::{AttributeKind, HasName, VariableDeclaration};
use fortitude_sitter::traits::HasNode;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks for public mutable module variables
///
/// ## Why is this bad?
/// Module variables that are accessible from outside their module can be modified
/// by any code that `use`s the module. This makes it hard to reason about their
/// value at any given point in a program, and can make testing and debugging
/// difficult: tests are no longer independent of each other, as state can leak
/// from one to the next.
///
/// Constants (those with the `parameter` attribute) are fine, as are variables
/// with the `protected` attribute, which cannot be modified outside the module
/// that declares them. Variables that are `private` are not flagged.
///
/// Instead of exposing a mutable variable, consider making it `private` and
/// providing procedures to get and set its value, or passing the value through
/// procedure arguments.
///
/// ## Examples
/// ```f90
/// module my_mod
///   implicit none (type, external)
///   private
///
///   public :: counter  ! Bad: any module that uses `my_mod` can change this
///   integer :: counter = 0
/// end module my_mod
/// ```
///
/// Use instead:
///
/// ```f90
/// module my_mod
///   implicit none (type, external)
///   private
///
///   public :: get_counter, increment_counter
///   integer :: counter = 0
///
/// contains
///   integer function get_counter()
///     get_counter = counter
///   end function get_counter
///
///   subroutine increment_counter()
///     counter = counter + 1
///   end subroutine increment_counter
/// end module my_mod
/// ```
///
/// or, if the variable should be visible but not modifiable from outside:
///
/// ```f90
/// module my_mod
///   implicit none (type, external)
///   private
///
///   integer, public, protected :: counter = 0
/// end module my_mod
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MutableModuleVariable {
    name: String,
}

impl Violation for MutableModuleVariable {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("Module variable '{name}' is public and mutable")
    }
}

/// Accessibility of module entities, as set by accessibility statements
#[derive(Default)]
struct Accessibility {
    /// `true` if a bare `private` statement is present
    default_private: bool,
    /// Names listed in `public :: ...` statements
    public: Vec<String>,
    /// Names listed in `private :: ...` statements
    private: Vec<String>,
}

impl Accessibility {
    fn from_module(module: &Node) -> Self {
        let mut access = Self::default();
        for child in module.named_children(&mut module.walk()) {
            let is_private = match child.kind_id() {
                kind!("private_statement") => true,
                kind!("public_statement") => false,
                _ => continue,
            };

            let names: Vec<String> = child
                .named_descendants()
                .filter(|node| matches!(node.kind_id(), kind!("identifier") | kind!("name")))
                .map(|node| node.text().to_ascii_lowercase())
                .collect();

            match (is_private, names.is_empty()) {
                // Bare `private`
                (true, true) => access.default_private = true,
                // Bare `public` is the default behaviour
                (false, true) => (),
                (true, false) => access.private.extend(names),
                (false, false) => access.public.extend(names),
            }
        }
        access
    }

    /// Is the variable with this name (lowercased) public, given that the
    /// declaration itself has no `public`/`private` attribute?
    fn is_public(&self, name: &str) -> bool {
        if self.public.iter().any(|n| n == name) {
            return true;
        }
        if self.private.iter().any(|n| n == name) {
            return false;
        }
        !self.default_private
    }
}

impl AstRule for MutableModuleVariable {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // Only variables declared directly in a module's specification part
        let module = node.parent()?;
        if !matches!(module.kind_id(), kind!("module") | kind!("submodule")) {
            return None;
        }

        let decl = VariableDeclaration::try_from_node(node).ok()?;

        // Constants and `protected` variables can't be modified from outside
        if decl.has_any_attributes(&[AttributeKind::Parameter, AttributeKind::Protected]) {
            return None;
        }

        // Explicitly private on the declaration
        if decl.has_attribute(AttributeKind::Private) {
            return None;
        }
        let explicitly_public = decl.has_attribute(AttributeKind::Public);

        let access = Accessibility::from_module(&module);

        let diagnostics = decl
            .names()
            .iter()
            .filter(|name| {
                explicitly_public || access.is_public(&name.name().as_str().to_ascii_lowercase())
            })
            .map(|name| {
                let name_text = name.name().as_str().to_string();
                context.create_diagnostic(Self { name: name_text }, name.node())
            })
            .collect::<Vec<_>>();

        Some(diagnostics)
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["variable_declaration"]
    }
}
