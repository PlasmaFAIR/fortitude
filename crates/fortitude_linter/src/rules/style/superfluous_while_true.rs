use crate::ast::FortitudeNode;
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::fix::edits::delete_stmt_part;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks for `while` statements that only evaluate the boolean literal `.true.` in `do`
/// statements.
///
/// ## Why is this bad?
/// The statement loop is superfluous, as it will always execute.
///
/// ## Example
/// ```f90
/// do while (.true.)
///   x = x + 1
///   if (x > 10) exit
/// end do
/// ```
///
/// Use instead:
/// ```f90
/// do
///   x = x + 1
///   if (x > 10) exit
/// end do
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct SuperfluousWhileTrue;

impl AlwaysFixableViolation for SuperfluousWhileTrue {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Superfluous boolean literal '.true.' in 'do while' statement".to_string()
    }

    fn fix_title(&self) -> String {
        "Remove 'while' statement".to_string()
    }
}

impl AstRule for SuperfluousWhileTrue {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        // TODO: delete "&" line returns if present inbetween the "do" and "while".

        // If for some reason a user has added reduntant parantheses, get rid of them.
        //   E.g `while ((.true.))` -> is equivalent to -> `while (.true.)` so we want to catch
        //   that as well.
        let parenth_exp = strip_redundant_parenths(*node);

        // We're left with the final parenthesised expression. It's child will always be a logical
        //   expression (which is valid and thus we return)
        //   or a single entity like a boolean literal.
        if !parenth_exp
            .child_with_name("boolean_literal")?
            .to_text(context.source_text())?
            .eq_ignore_ascii_case(".true.")
        {
            return None;
        }
        let edits = delete_stmt_part(node, context.source_text());
        let mut iter = edits.into_iter();
        let fix = Fix::safe_edits(iter.next().expect("edits should be nonempty"), iter);
        some_vec!(context.create_diagnostic(Self {}, node).with_fix(fix))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["while_statement"]
    }
}

fn strip_redundant_parenths(mut node: Node) -> Node {
    while let Some(inner) = node.child_with_name("parenthesized_expression") {
        node = inner;
    }

    node
}
