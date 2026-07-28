use crate::ast::FortitudeNode;
use crate::ast::types::{AttributeKind, HasName, Intent, Type};
use crate::diagnostics::{Annotation, Diagnostic, Span, Violation};
use crate::settings::FortranStandard;
use crate::traits::HasNode;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use itertools::Itertools;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for assumed size variables
///
/// ## Why is this bad?
/// Assumed size dummy arguments declared with a star `*` as the size should be
/// avoided. There are several downsides to assumed size, the main one being
/// that the compiler is not able to determine the array bounds, so it is not
/// possible to check for array overruns or to use the array in whole-array
/// expressions.
///
/// Instead, prefer assumed shape arguments, as the compiler is able to keep track of
/// the upper bounds automatically, and pass this information under the hood. It also
/// allows use of whole-array expressions, such as `a = b + c`, where `a, b, c` are
/// all arrays of the same shape.
///
/// Instead of:
///
/// ```f90
/// subroutine process_array(array)
///     integer, dimension(*), intent(in) :: array
///     ...
/// ```
///
/// use:
///
/// ```f90
/// subroutine process_array(array)
///     integer, dimension(:), intent(in) :: array
///     ...
/// ```
///
/// Note that this doesn't apply to `character` types, where `character(len=*)` is
/// actually the most appropriate specification for `intent(in)` arguments! This is
/// because `character(len=:)` must be either a `pointer` or `allocatable`.
#[derive(ViolationMetadata)]
pub(crate) struct AssumedSize {
    name: String,
}

impl Violation for AssumedSize {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("'{name}' has assumed size")
    }
}
impl AstRule for AssumedSize {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let src = context.source_text();
        let declaration = context
            .symbol_table()
            .current()?
            .decl_containing_node(node)?;

        // Deal with `character([len=]*)` elsewhere
        if let Type::Intrinsic(type_) = declaration.type_()
            && type_.is_character()
            && node
                .ancestors()
                .any(|parent| parent.kind_id() == kind!("kind"))
        {
            return None;
        }

        // Assumed size ok for parameters
        if declaration.has_attribute(AttributeKind::Parameter) {
            return None;
        }

        // Are we looking at something declared like `array(*)`?
        if let Some(sized_decl) = node
            .ancestors()
            .find(|parent| parent.kind_id() == kind!("sized_declarator"))
        {
            let identifier = sized_decl.child_with_id(kind!("identifier"))?;
            let name = identifier.to_text(src)?.to_string();
            return some_vec![context.create_diagnostic(Self { name }, node)];
        }

        // Collect things that look like `dimension(*)` -- this
        // applies to all identifiers on this line
        let all_decls = declaration
            .names()
            .iter()
            .map(|name| name.name().to_string())
            .map(|name| context.create_diagnostic(Self { name }, node))
            .collect_vec();

        Some(all_decls)
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["assumed_size"]
    }
}

/// ## What does it do?
/// Checks `character` dummy arguments with an assumed-size length have
/// `intent(in)` only.
///
/// ## Why is this bad?
/// Character dummy arguments whose length is assumed size should only have
/// `intent(in)`, as this can cause data loss with `intent([in]out)`. For
/// example:
///
/// ```f90
/// program example
///   character(len=3) :: short_text
///   call set_text(short_text)
///   print*, short_text
/// contains
///   subroutine set_text(text)
///     character(*), intent(out) :: text
///     text = "longer than 3 characters"
///   end subroutine set_text
/// end program
/// ```
///
/// Here, `short_text` will only contain the truncated "lon".
///
/// To handle dynamically setting `character` sizes, use `allocatable` instead:
///
/// ```f90
/// program example
///   character(len=:), allocatable :: allocatable_text
///   call set_text(allocatable_text)
///   print*, allocatable_text
/// contains
///   subroutine set_text(text)
///     character(len=:), allocatable, intent(out) :: text
///     text = "longer than 3 characters"
///   end subroutine set_text
/// end program
/// ```
///
/// Allocatable dummy arguments were not introduced until Fortran 2003, so this
/// rule is deactivated when targeting earlier standards. When doing so, it is
/// recommended to always verify that the `character` dummy arguments have the
/// correct size to avoid data loss:
///
/// ```f90
///   ! Fortran 95 example
///   subroutine set_text(text)
///     character(len=*), intent(out) :: text
///     if (len(text) < 12) stop 1
///     text = "hello world!"
///   end subroutine set_text
/// ```
///
/// ## User derived type IO procedures
/// The standard mandates assumed-size length with `intent(inout)` for the
/// `iomsg` argument of user defined IO procedures for derived types, although
/// it doesn't specify a minimum length. Unfortunately, Fortitude is currently
/// unable to detect this use. You can use [`allow` (suppression)
/// comments](https://fortitude.readthedocs.io/en/stable/linter/#error-suppression)
/// to disable this rule for those uses only.
#[derive(ViolationMetadata)]
pub(crate) struct AssumedSizeCharacterIntent {
    name: String,
}

impl Violation for AssumedSizeCharacterIntent {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("character '{name}' has assumed size but does not have `intent(in)`")
    }
}
impl AstRule for AssumedSizeCharacterIntent {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // The recommended fix to this is only possible in Fortran 2003 and later.
        // Those still writing Fortran 95 code are on their own!
        if context.settings().target_std < FortranStandard::F2003 {
            return None;
        }

        // TODO: This warning will also catch:
        // - non-dummy arguments -- these are always invalid, should be a separate warning?

        // Find the declaration containing this node
        let current_context = context.symbol_table().current()?;
        let declaration = current_context.decl_containing_node(node)?;

        // Only applies to `character`
        if let Type::Intrinsic(type_) = declaration.type_()
            && !type_.is_character()
        {
            return None;
        }

        // Handle `character*(*)` elsewhere -- note this just skips emitting a warning
        // for the first `*`, we'll still get one for the second `*`, but this is desired
        if let Some(sibling) = node.next_named_sibling()
            && sibling.kind_id() == kind!("assumed_size")
        {
            return None;
        }

        // Assumed size ok for parameters and `intent(in)` only
        if declaration
            .has_any_attributes(&[AttributeKind::Parameter, AttributeKind::Intent(Intent::In)])
        {
            return None;
        }

        // Collect all declarations on this line
        Some(
            declaration
                .names()
                .iter()
                .map(|name| {
                    let annotation = Annotation::secondary(
                        Span::from(context.source_file().clone()).with_range(name.node()),
                    );
                    let name = name.name().to_string();
                    let mut diagnostic = context.create_diagnostic(Self { name }, node);
                    diagnostic.annotate(annotation);
                    diagnostic
                })
                .collect_vec(),
        )
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["assumed_size"]
    }
}
