use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What it does
/// Checks that entities imported by a `use ... only:` statement are sorted.
///
/// ## Why is this bad?
/// Sorted imports are easier to scan, reduce cognitive load when reviewing
/// code, and help avoid merge conflicts when multiple developers add entities
/// to the same statement.
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use my_module, only: zeta, alpha, beta
///
/// ! Better
/// use my_module, only: alpha, beta, zeta
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct UnsortedUseStatement;

impl Violation for UnsortedUseStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`use` statement imports are not sorted".to_string()
    }
}

impl AstRule for UnsortedUseStatement {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let included_items = node.child_with_id(kind!("included_items"))?;
        let mut previous = None;
        let mut is_unsorted = false;

        for item in included_items.named_children(&mut included_items.walk()) {
            let name = match item.kind() {
                "identifier" => item.text().to_ascii_lowercase(),
                "use_alias" => item
                    .child(0)
                    .map(|local_name| local_name.text().to_ascii_lowercase())?,
                _ => continue,
            };

            if let Some(previous) = &previous
                && name < *previous
            {
                is_unsorted = true;
            }
            previous = Some(name);
        }

        if is_unsorted {
            some_vec!(context.create_diagnostic(Self {}, node))
        } else {
            None
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["use_statement"]
    }
}
