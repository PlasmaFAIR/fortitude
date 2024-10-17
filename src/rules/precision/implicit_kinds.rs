use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;
/// Defines rules that require the user to explicitly specify the kinds of any reals.
pub struct ImplicitRealKind {}

impl Rule for ImplicitRealKind {
    fn new(_settings: &Settings) -> Self {
        ImplicitRealKind {}
    }

    fn explain(&self) -> &'static str {
        "
        Real variable declarations without an explicit kind will have a compiler/platform
        dependent precision, which hurts portability and may lead to surprising loss of
        precision in some cases.
        "
    }
}

impl ASTRule for ImplicitRealKind {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let dtype = node.child(0)?.to_text(src.source_text())?.to_lowercase();

        if !matches!(dtype.as_str(), "real" | "complex") {
            return None;
        }

        if node.child_by_field_name("kind").is_some() {
            return None;
        }

        let msg = format!("{dtype} has implicit kind");
        some_vec![Violation::from_node(msg, node)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
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

        let expected: Vec<Violation> = [
            (1, 0, 1, 4, "real"),
            (2, 2, 2, 6, "real"),
            (5, 2, 5, 9, "complex"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, dtype)| {
            Violation::from_start_end_line_col(
                format!("{dtype} has implicit kind"),
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
