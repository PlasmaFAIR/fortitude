use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks that consecutive procedure definitions are separated by a blank line.
///
/// ## Why is this bad?
/// A blank line between procedures makes it easier to distinguish where one
/// procedure ends and the next begins.
///
/// ## Example
/// ```f90
/// module my_module
///   implicit none
/// contains
///   subroutine foo()
///     print *, "foo"
///   end subroutine foo
///   subroutine bar()
///     print *, "bar"
///   end subroutine bar
/// end module my_module
/// ```
///
/// Use instead:
/// ```f90
/// module my_module
///   implicit none
/// contains
///   subroutine foo()
///     print *, "foo"
///   end subroutine foo
///
///   subroutine bar()
///     print *, "bar"
///   end subroutine bar
/// end module my_module
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MissingBlankLineBetweenProcedures;

impl Violation for MissingBlankLineBetweenProcedures {
    #[derive_message_formats]
    fn message(&self) -> String {
        "procedure definitions should be separated by a blank line".to_string()
    }
}

impl AstRule for MissingBlankLineBetweenProcedures {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut previous: Option<Node> = None;

        for child in node.named_children(&mut node.walk()) {
            if !matches!(child.kind(), "function" | "subroutine") {
                continue;
            }

            if let Some(previous) = previous {
                let between = &context.source_text()[previous.end_byte()..child.start_byte()];
                let has_blank_line = between
                    .split_inclusive('\n')
                    .filter(|line| line.ends_with('\n'))
                    .any(|line| line.trim().is_empty());
                if !has_blank_line {
                    let statement = child.child(0).unwrap_or(child);
                    diagnostics.push(context.create_diagnostic(Self {}, statement));
                }
            }

            previous = Some(child);
        }

        if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["translation_unit", "internal_procedures"]
    }
}
