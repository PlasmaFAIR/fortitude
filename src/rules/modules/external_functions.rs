use crate::{Method, Rule, Violation};
use tree_sitter::Node;

/// Defines rules that check whether functions and subroutines are defined within modules (or one
/// of a few acceptable alternatives).

fn external_function(node: &Node, _src: &str) -> Option<Violation> {
    if node.parent()?.kind() == "translation_unit" {
        let msg = format!(
            "{} not contained within (sub)module or program",
            node.kind()
        );
        return Some(Violation::from_node(msg, node));
    }
    None
}

pub struct ExternalFunction {}

impl Rule for ExternalFunction {
    fn method(&self) -> Method {
        Method::Tree(external_function)
    }

    fn explain(&self) -> &str {
        "
        Functions and subroutines should be contained within (sub)modules or programs.
        Fortran compilers are unable to perform type checks and conversions on functions
        defined outside of these scopes, and this is a common source of bugs.
        "
    }

    fn entrypoints(&self) -> Vec<&str> {
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
        let actual = ExternalFunction {}.apply(&source.as_str(), &default_settings())?;
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
        let actual = ExternalFunction {}.apply(&source.as_str(), &default_settings())?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
