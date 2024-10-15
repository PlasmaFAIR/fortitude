use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Defines rules that check whether functions and subroutines are defined within modules (or one
/// of a few acceptable alternatives).

pub struct ExternalFunction {}

impl Rule for ExternalFunction {
    fn new(_settings: &Settings) -> Self {
        ExternalFunction {}
    }

    fn explain(&self) -> &'static str {
        "
        Functions and subroutines should be contained within (sub)modules or programs.
        Fortran compilers are unable to perform type checks and conversions on functions
        defined outside of these scopes, and this is a common source of bugs.
        "
    }
}

impl ASTRule for ExternalFunction {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<Violation>> {
        if node.parent()?.kind() == "translation_unit" {
            let msg = format!(
                "{} not contained within (sub)module or program",
                node.kind()
            );
            let procedure_stmt = node.child(0)?;
            return some_vec![Violation::from_node(msg, &procedure_stmt)];
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["function", "subroutine"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_function_not_in_module() -> anyhow::Result<()> {
        let source = test_file(
            "
            integer function double(x)
              integer, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              integer, intent(inout) :: x
              x = 3 * x
            end subroutine
            ",
        );
        let expected: Vec<Violation> = [(1, 0, 1, 26, "function"), (6, 0, 6, 20, "subroutine")]
            .iter()
            .map(|(start_line, start_col, end_line, end_col, kind)| {
                Violation::from_start_end_line_col(
                    format!("{kind} not contained within (sub)module or program"),
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            })
            .collect();
        let rule = ExternalFunction::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_function_in_module() -> anyhow::Result<()> {
        let source = test_file(
            "
            module my_module
                implicit none
            contains
                integer function double(x)
                  integer, intent(in) :: x
                  double = 2 * x
                end function

                subroutine triple(x)
                  integer, intent(inout) :: x
                  x = 3 * x
                end subroutine
            end module
            ",
        );
        let expected: Vec<Violation> = vec![];
        let rule = ExternalFunction::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
