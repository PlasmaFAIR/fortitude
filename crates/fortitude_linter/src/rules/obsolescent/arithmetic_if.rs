use crate::settings::CheckSettings;
use crate::{AstRule, FromAstNode, SymbolTables};
use ruff_diagnostics::{Diagnostic, Fix, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for arithmetic `if` statements.
///
/// ## Why is this bad?
/// The arithmetic `if` statement is used to jump between one of three statement
/// labels depending on whether the condition is below, above, or equal to
/// zero. However, this is incompatible with the IEEE 754 standard on floating
/// point numbers (due to the comparison between `real`s), and the use of
/// statment labels can hinder optimisation, as well as making the code harder
/// to read and maintain.
///
/// ## Example
/// ```f90
///     IF(x(1)) 10, 20, 30
/// 10  PRINT *, 'first case'
///     GOTO 40
/// 20  PRINT *, 'second case'
///     GOTO 40
/// 30  PRINT *, 'third case'
/// 40  CONTINUE
/// ```
///
/// Use instead:
/// ```f90
/// if (x(1) < 0) then
///   print*, "first case"
/// else if (x(1) > 0) then
///   print*, "third case"
/// else
///   print*, "second case"
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct ArithmeticIf;

impl Violation for ArithmeticIf {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Obsolete arithmetic `if`".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `if` statement or `if` construct".into())
    }
}

/// We can fix if:
/// 1. labels follow the `if` (no going backwards
/// 2. the previous non-comment statement to each label is one of:
///    - the `if` statement itself
///    - `goto`
///    - `return`
/// 3. if the previous non-comment statement is a `goto` then:
///    - it must follow the last label
///
/// This give us structures like:
///
/// if (condition) A, B, C
/// <only comments>
/// A <statement>
///   ...
///   (goto D | return)
/// B <statement>
///   ...
///   (goto D | return)
/// C <statement>
///   ...
/// [D <statement>]
///
/// A, B, C don't have to appear in that order!
///
/// If A, B, C are all distinct, and either A or B blocks end in `goto
/// D`, then this should get translated to:
///
/// if (condition < 0) then
///    A ...
/// else if (condition > 0) then
///    C ...
/// else
///    B ...
/// end if
///
/// If A, B are identical, then `condition` should be translated to `<=` and we don't need B
/// If B, C are identical, then `condition` should be translated to `>=` and we don't need B
///
/// If either A or B end in `return`, then we must be a lot more careful,
/// because we may not know where C ends.
/// It must end with the close of the current block -- do we have a way of getting that?
fn fix_arithmetic_if(node: &Node) -> Option<Fix> {
    None
}

impl AstRule for ArithmeticIf {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        _src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let mut diagnostic = Diagnostic::from_node(ArithmeticIf {}, node);
        if let Some(fix) = fix_arithmetic_if(node) {
            diagnostic.set_fix(fix);
        }
        some_vec![diagnostic]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["arithmetic_if_statement"]
    }
}
