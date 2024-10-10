use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
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
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        let dtype = node.child(0)?.to_text(src)?.to_lowercase();

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
    use crate::settings::default_settings;
    use crate::violation;
    use pretty_assertions::assert_eq;
    use textwrap::dedent;

    #[test]
    fn test_implicit_real_kind() -> anyhow::Result<()> {
        let source = dedent(
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

        let expected: Vec<Violation> = [(2, 1, "real"), (3, 3, "real"), (6, 3, "complex")]
            .iter()
            .map(|(line, col, dtype)| {
                let msg = format!("{dtype} has implicit kind");
                violation!(&msg, *line, *col)
            })
            .collect();

        let rule = ImplicitRealKind::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
