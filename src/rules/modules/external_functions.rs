use crate::settings::Settings;
use crate::{ASTRule, FortitudeViolation, Rule};
use ruff_diagnostics::Violation;
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks whether functions and subroutines are defined within modules (or one
/// of a few acceptable alternatives).
///
/// ## Why is this bad?
/// Functions and subroutines should be contained within (sub)modules or programs.
/// Fortran compilers are unable to perform type checks and conversions on functions
/// defined outside of these scopes, and this is a common source of bugs.
#[violation]
pub struct ExternalFunction {
    procedure: String,
}

impl Violation for ExternalFunction {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ExternalFunction { procedure } = self;
        format!("{procedure} not contained within (sub)module or program")
    }
}

impl Rule for ExternalFunction {
    fn new(_settings: &Settings) -> Self {
        ExternalFunction {
            procedure: String::default(),
        }
    }
}

impl ASTRule for ExternalFunction {
    fn check(&self, node: &Node, _src: &SourceFile) -> Option<Vec<FortitudeViolation>> {
        if node.parent()?.kind() == "translation_unit" {
            let msg = ExternalFunction {
                procedure: node.kind().to_string(),
            }
            .message();
            let procedure_stmt = node.child(0)?;
            return some_vec![FortitudeViolation::from_node(msg, &procedure_stmt)];
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
        let expected: Vec<FortitudeViolation> =
            [(1, 0, 1, 26, "function"), (6, 0, 6, 20, "subroutine")]
                .iter()
                .map(|(start_line, start_col, end_line, end_col, kind)| {
                    let msg = ExternalFunction {
                        procedure: kind.to_string(),
                    }
                    .message();
                    FortitudeViolation::from_start_end_line_col(
                        msg,
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
        let expected: Vec<FortitudeViolation> = vec![];
        let rule = ExternalFunction::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
