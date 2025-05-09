use crate::settings::Settings;
use crate::AstRule;
use crate::{ast::FortitudeNode, FromAstNode};
use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextRange;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of a variable in the same logical expression as "definedness" inquiry.
///
/// ## Why is this bad?
/// Unlike many other languages, the Fortran standard doesn't mandate (or prohibit)
/// short-circuiting in logical expressions, so different compilers have different
/// behaviour when it comes to evaluating such expressions. This is commonly encountered
/// when using `present()` with an optional dummy argument and checking its value in the
/// same expression. Without short-circuiting, this can lead to segmentation faults when
/// the expression is evaluated if the argument isn't present.
///
/// Instead, you should nest the conditional statements, or use the Fortran 2023
/// "condtional expression" (also called ternary expressions in other
/// languages). Unfortunately, any `else` branches may need to be duplicated or
/// refactored to accommodate this change.
///
/// This lack of short-circuiting also affects other inquiry functions such as
/// `associated` and `allocated` which are used to guard invalid accesses.
///
/// ## Example
/// Don't do this:
/// ```f90
/// integer function example(arg1)
///   integer, optional, intent(in) :: arg1
///
///   if (present(arg1) .and. arg1 > 2) then
///     example = arg1 * arg1
///   else
///     example = 1
///   end if
/// ```
/// The compiler may or may not evaluate `arg1 > 2` _even if_ `present(arg1)` is
/// false. This is a runtime error, and may crash your program.
///
/// Use instead, noting that we either need to duplicate the `else` branch, or refactor
/// differently:
/// ```f90
/// integer function example(arg1)
///   integer, optional, intent(in) :: arg1
///
///   if (present(arg1)) then
///     if (arg1 > 2) then
///       example = arg1 * arg1
///     else
///       example = 1
///     end if
///   else
///     example = 1
///   end if
/// ```
///
/// Or with Fortran 2023 (not currently supported by most compilers!):
/// ```f90
/// integer function example(arg1)
///   integer, optional, intent(in) :: arg1
///
///   example = present(arg1) ? (arg1 > 2 ? arg1 * arg1 : 1) : 1
/// ```
/// Note that although the true/false arms of the conditional-expression are lazily
/// evaluated, it's still not possible to use a compound logical expression here, so we
/// still must have a nested expression and duplicate the default value.
///
/// ## References
/// - <https://www.scivision.dev/fortran-short-circuit-logic/>
#[derive(ViolationMetadata)]
pub(crate) struct NonportableShortcircuitInquiry {
    arg: String,
    function: String,
    // Useful for when we get multiple notes in DiagnosticMessages
    #[allow(dead_code)]
    present: TextRange,
}

impl Violation for NonportableShortcircuitInquiry {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { arg, function, .. } = self;
        format!("variable inquiry `{function}({arg})` and use in same logical expression")
    }
}

#[derive(Debug)]
struct PresentCall {
    pub range: TextRange,
    pub arg: String,
}

fn present_call(expr: &Node, src: &str, function: &str) -> Option<PresentCall> {
    if expr.kind() != "call_expression" {
        return None;
    }
    if expr.child(0)?.to_text(src)?.to_lowercase() != function {
        return None;
    }
    let arg_list = expr.child_with_name("argument_list")?;
    // Make sure we skip the two-arg version of `associated`
    // Length 3: "(", "identifier, ")"
    if arg_list.children(&mut arg_list.walk()).len() != 3 {
        return None;
    }

    let identifier = arg_list.child_with_name("identifier")?;
    let arg = identifier.to_text(src)?.to_lowercase().to_string();

    Some(PresentCall {
        range: expr.textrange(),
        arg,
    })
}

impl AstRule for NonportableShortcircuitInquiry {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let expr = node.child(1)?;
        let text = src.source_text();

        let violations = ["present", "allocated", "associated"]
            .iter()
            .flat_map(|function| find_nonportable_shortcircuits(&expr, text, function))
            .collect_vec();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["if_statement"]
    }
}

fn find_nonportable_shortcircuits(node: &Node, text: &str, function: &str) -> Vec<Diagnostic> {
    // First find all the `present(foo)` calls
    let calls = node
        .descendants()
        .filter_map(|e| present_call(&e, text, function))
        .collect_vec();

    // Now check if any identifier appears in this `if` statement
    // that is an argument of a `present()` call
    node.descendants()
        .filter(|node| {
            // Filter out any nodes that overlap one of the
            // `present()` calls we found. This stops us catching
            // multiple `present()` calls with the same argument
            !calls
                .iter()
                .any(|call| call.range.contains(node.start_textsize()))
        })
        .filter(|expr| expr.kind() == "identifier")
        .filter_map(|expr| {
            // Now check if this identifier matches any in the `present()` calls
            let id = expr.to_text(text).unwrap_or_default().to_lowercase();
            if let Some(PresentCall { range, .. }) = calls.iter().find(|call| call.arg == id) {
                Some((expr, id, range))
            } else {
                None
            }
        })
        .map(|(node, arg, &present)| {
            Diagnostic::from_node(
                NonportableShortcircuitInquiry {
                    arg,
                    function: function.to_string(),
                    present,
                },
                &node,
            )
        })
        .collect_vec()
}
