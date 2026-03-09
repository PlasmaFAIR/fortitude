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
/// - `check.complexity.max-complexity`
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
        let max_complexity = settings.complexity.max_complexity;

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

/// ## What it does
/// Checks for procedures with a large number of arguments.
///
/// ## Why is this bad?
/// Procedures with many arguments are harder to understand, maintain, call,
/// and test. They often indicate that a procedure is doing too much and should
/// be refactored into smaller, more focused procedures, or that a derived type
/// should be used to group related arguments together.
///
/// The equivalent rule in Pylint has a default threshold of 5, but we set it to
/// 10 for Fortran to account for the fact that Fortran procedures often have
/// more arguments than Python functions.
///
/// ## Example
///
/// If we set the threshold to 6 or lower, then the following procedure would be
/// flagged for having too many arguments:
/// ```f90
/// subroutine update_position(x, y, z, vx, vy, vz, dt)
///   real, intent(inout) :: x, y, z
///   real, intent(in) :: vx, vy, vz, dt
///   x = x + vx * dt
///   y = y + vy * dt
///   z = z + vz * dt
/// end subroutine update_position
/// ```
///
/// Use instead:
/// ```f90
/// subroutine update_position(position, velocity, dt)
///   type(vector), intent(inout) :: position
///   type(vector), intent(in) :: velocity
///   real, intent(in) :: dt
///   position%x = position%x + velocity%x * dt
///   position%y = position%y + velocity%y * dt
///   position%z = position%z + velocity%z * dt
/// end subroutine update_position
/// ```
///
/// where `vector` is a derived type defined as:
/// ```f90
/// type :: vector
///  real :: x
///  real :: y
///  real :: z
/// end type vector
/// ```
///
/// ## Options
/// - `check.complexity.max-args`
#[derive(ViolationMetadata)]
pub(crate) struct TooManyArguments {
    actual_args: usize,
    max_args: usize,
    procedure_name: String,
}

impl Violation for TooManyArguments {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            actual_args,
            max_args,
            procedure_name,
        } = self;
        format!("Too many arguments in procedure `{procedure_name}` ({actual_args} > {max_args})")
    }
}

impl AstRule for TooManyArguments {
    fn check<'a>(
        settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        let procedure_stmt = node.named_child(0)?;
        let procedure_name = procedure_stmt
            .child_with_name("name")?
            .to_text(src)?
            .to_string();
        let parameters = procedure_stmt.child_with_name("parameters")?;
        let actual_args = parameters
            .named_descendants()
            .filter(|node| node.kind() == "identifier")
            .count();
        let max_args = settings.complexity.max_args;

        if actual_args > max_args {
            return some_vec![Diagnostic::from_node(
                TooManyArguments {
                    actual_args,
                    max_args,
                    procedure_name,
                },
                &parameters,
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::Display;

    #[derive(Debug, Clone, CacheKey)]
    pub struct Settings {
        pub max_complexity: usize,
        pub max_args: usize,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self {
                max_complexity: 10,
                max_args: 10,
            }
        }
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.complexity",
                fields = [self.max_complexity, self.max_args]
            }
            Ok(())
        }
    }
}
