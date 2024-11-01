use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, FromASTNode, Rule};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks that `real` variables have their kind explicitly specified
///
/// ## Why is this bad?
/// Real variable declarations without an explicit kind will have a compiler/platform
/// dependent precision, which hurts portability and may lead to surprising loss of
/// precision in some cases.
#[violation]
pub struct ImplicitRealKind {
    dtype: String,
}

impl Violation for ImplicitRealKind {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ImplicitRealKind { dtype } = self;
        format!("{dtype} has implicit kind")
    }
}

impl Rule for ImplicitRealKind {
    fn new(_settings: &Settings) -> Self {
        ImplicitRealKind {
            dtype: String::default(),
        }
    }
}

impl ASTRule for ImplicitRealKind {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let dtype = node.child(0)?.to_text(src.source_text())?.to_lowercase();

        if !matches!(dtype.as_str(), "real" | "complex") {
            return None;
        }

        if node.child_by_field_name("kind").is_some() {
            return None;
        }

        some_vec![Diagnostic::from_node(ImplicitRealKind { dtype }, node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file, FromStartEndLineCol};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_implicit_real_kind() -> anyhow::Result<()> {
        let source = test_file(
            "
            real function my_func(a, b, c, d, e)       ! catch
              real, intent(in) :: a                    ! catch
              real(4), intent(in) :: b                 ! ignore
              integer, intent(in) :: c                 ! ignore
              complex, intent(in) :: d                 ! catch
              complex(8), intent(in) :: e              ! ignore

              myfunc = a
            end function my_func
            ",
        );

        let expected: Vec<_> = [
            (1, 0, 1, 4, "real"),
            (2, 2, 2, 6, "real"),
            (5, 2, 5, 9, "complex"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, dtype)| {
            Diagnostic::from_start_end_line_col(
                ImplicitRealKind {
                    dtype: dtype.to_string(),
                },
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();

        let rule = ImplicitRealKind::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
