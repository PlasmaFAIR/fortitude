use crate::ast::FortitudeNode;
use crate::diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use crate::{AstRule, CheckContext};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for old style array literals
///
/// ## Why is this bad?
/// Fortran 2003 introduced a shorter syntax for array literals: `[...]`. While the
/// older style, `(/.../)`, is still valid, the F2003 style is shorter and easier to
/// match.
#[derive(ViolationMetadata)]
pub(crate) struct OldStyleArrayLiteral {}

impl AlwaysFixableViolation for OldStyleArrayLiteral {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Array literal uses old-style syntax: prefer `[...]`".to_string()
    }

    fn fix_title(&self) -> String {
        "Change `(/.../)` to `[...]`".to_string()
    }
}

impl AstRule for OldStyleArrayLiteral {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let open_bracket = node.child(0)?;

        if open_bracket
            .to_text(context.source_text())?
            .starts_with("(/")
        {
            let close_bracket = node.children(&mut node.walk()).last()?;
            let src = context.source_file();
            let edit_open = open_bracket.edit_replacement(src, "[".to_string());
            let edit_close = close_bracket.edit_replacement(src, "]".to_string());
            let fix = Fix::safe_edits(edit_open, [edit_close]);

            return some_vec!(context.create_diagnostic(Self {}, node).with_fix(fix));
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["array_literal"]
    }
}
