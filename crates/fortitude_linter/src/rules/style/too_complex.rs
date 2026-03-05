use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for procedures with a cyclomatic complexity that exceeds a
/// configurable threshold.
///
/// ## Why is this bad?
/// Cyclomatic complexity measures the number of linearly independent paths
/// through a procedure. A high complexity indicates that a procedure has too
/// many branches, making it harder to read, test, and maintain. Procedures
/// with a complexity above 10 (the threshold proposed by McCabe) are
/// generally considered too complex and should be refactored.
///
/// ## Example
/// ```f90
/// subroutine classify(x, category)
///   real, intent(in) :: x
///   integer, intent(out) :: category
///   if (x < 0.0) then
///     if (x < -100.0) then
///       category = 1
///     else if (x < -10.0) then
///       category = 2
///     else
///       category = 3
///     end if
///   else if (x == 0.0) then
///     category = 4
///   else
///     if (x > 100.0) then
///       category = 5
///     else if (x > 10.0) then
///       category = 6
///     else
///       category = 7
///     end if
///   end if
/// end subroutine classify
/// ```
///
/// Use instead:
/// ```f90
/// integer function classify_negative(x)
///   real, intent(in) :: x
///   if (x < -100.0) then
///     classify_negative = 1
///   else if (x < -10.0) then
///     classify_negative = 2
///   else
///     classify_negative = 3
///   end if
/// end function classify_negative
///
/// integer function classify_positive(x)
///   real, intent(in) :: x
///   if (x > 100.0) then
///     classify_positive = 5
///   else if (x > 10.0) then
///     classify_positive = 6
///   else
///     classify_positive = 7
///   end if
/// end function classify_positive
///
/// subroutine classify(x, category)
///   real, intent(in) :: x
///   integer, intent(out) :: category
///   if (x < 0.0) then
///     category = classify_negative(x)
///   else if (x == 0.0) then
///     category = 4
///   else
///     category = classify_positive(x)
///   end if
/// end subroutine classify
/// ```
///
/// ## Options
/// - `check.too-complex.max-complexity`
///
/// ## References
/// - [Wikipedia: Cyclomatic complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity)
#[derive(ViolationMetadata)]
pub(crate) struct TooComplex {
    actual_complexity: usize,
    max_complexity: usize,
}

impl Violation for TooComplex {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            actual_complexity,
            max_complexity,
        } = self;
        format!("cyclomatic complexity of {actual_complexity}, exceeds maximum {max_complexity}")
    }
}

impl AstRule for TooComplex {
    fn check<'a>(
        settings: &CheckSettings,
        node: &'a Node,
        _src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let procedure_stmt = node.named_child(0)?;
        let actual_complexity = cyclomatic_complexity(node);
        let max_complexity = settings.too_complex.max_complexity;

        if actual_complexity > max_complexity {
            return some_vec![Diagnostic::from_node(
                TooComplex {
                    actual_complexity,
                    max_complexity,
                },
                &procedure_stmt,
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["program", "function", "subroutine"]
    }
}

fn cyclomatic_complexity(node: &Node) -> usize {
    // Start at 1 (base path through the procedure)
    let mut complexity = 1;

    for child in node.descendants() {
        match child.kind() {
            // if-then and else-if each create an independent branch
            "if_statement" | "elseif_clause" => complexity += 1,
            // Each case branch counts individually; default is excluded
            // as it does not create an independent path
            "case_statement" => {
                let is_default = child
                    .named_child(0)
                    .map(|c| c.kind() == "default")
                    .unwrap_or(false);
                if !is_default {
                    complexity += 1;
                }
            }
            // do and do-while loops both appear as do_loop in the AST
            "do_loop" => complexity += 1,
            // where construct is an array-level conditional branch
            "where_statement" => complexity += 1,
            // Each .and. and .or. operator creates an additional path
            // (each binary logical_expression node = one operator)
            "logical_expression" => complexity += 1,
            _ => {}
        }
    }

    complexity
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::Display;

    #[derive(Debug, Clone, CacheKey)]
    pub struct Settings {
        pub max_complexity: usize,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self { max_complexity: 10 }
        }
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.too-complex",
                fields = [self.max_complexity]
            }
            Ok(())
        }
    }
}
