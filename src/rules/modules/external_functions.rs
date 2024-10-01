use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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
    fn check(&self, node: &Node, _src: &str) -> Option<Vec<Violation>> {
        if node.parent()?.kind() == "translation_unit" {
            let msg = format!(
                "{} not contained within (sub)module or program",
                node.kind()
            );
            return some_vec![Violation::from_node(msg, node)];
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
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_function_not_in_module() -> anyhow::Result<()> {
        let source = dedent(
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
        let expected: Vec<Violation> = [(2, 1, "function"), (7, 1, "subroutine")]
            .iter()
            .map(|(line, col, kind)| {
                let msg = format!("{} not contained within (sub)module or program", kind);
                violation!(&msg, *line, *col)
            })
            .collect();
        let rule = ExternalFunction::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_function_in_module() -> anyhow::Result<()> {
        let source = dedent(
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
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
