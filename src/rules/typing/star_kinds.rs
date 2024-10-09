use crate::ast::{dtype_is_plain_number, strip_line_breaks, FortitudeNode};
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use tree_sitter::Node;
/// Defines rules that discourage the use of the non-standard kind specifiers such as
/// `int*4` or `real*8`. Also prefers the use of `character(len=*)` to
/// `character*(*)`, as although the latter is permitted by the standard, the former is
/// more explicit.

pub struct StarKind {}

impl Rule for StarKind {
    fn new(_settings: &Settings) -> Self {
        StarKind {}
    }

    fn explain(&self) -> &'static str {
        "
        Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
        avoided. For these cases, consider instead using 'real(real64)' or
        'integer(int32)', where 'real64' and 'int32' may be found in the intrinsic
        module 'iso_fortran_env'. You may also wish to determine kinds using the
        built-in functions 'selected_real_kind' and 'selected_int_kind'.
        "
    }
}

impl ASTRule for StarKind {
    fn check(&self, node: &Node, src: &str) -> Option<Vec<Violation>> {
        let dtype = node.child(0)?.to_text(src)?.to_lowercase();
        // TODO: Handle characters
        if !dtype_is_plain_number(dtype.as_str()) {
            return None;
        }
        let kind_node = node.child_by_field_name("kind")?;
        let size = kind_node.to_text(src)?;
        if !size.starts_with('*') {
            return None;
        }

        // Tidy up the kind spec so it's just e.g. '*8'
        let size = strip_line_breaks(size).replace([' ', '\t'], "");

        let literal = kind_node.child_with_name("number_literal")?;
        let kind = literal.to_text(src)?;
        // TODO: Better suggestion, rather than use integer literal
        let msg = format!("{dtype}{size} is non-standard, use {dtype}({kind})");
        some_vec![Violation::from_node(msg, &kind_node)]
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
    fn test_star_kind() -> anyhow::Result<()> {
        let source = dedent(
            "
            integer*8 function add_if(x, y, z)
              integer(kind=2), intent(in) :: x
              integer *4, intent(in) :: y
              logical*   4, intent(in) :: z
              real    * &
               8 :: t

              if (x == 2) then
                add_if = x + y
              else
                add_if = x
              end if
            end function

            subroutine complex_mul(x, real)
              real * 4, intent(in) :: x
              complex  *  8, intent(inout) :: real
              ! This would be a false positive with purely regexp based linting
              real = real * 8
            end subroutine
            ",
        );

        let expected: Vec<Violation> = [
            (2, 8, "integer*8", "integer(8)"),
            (4, 11, "integer*4", "integer(4)"),
            (5, 10, "logical*4", "logical(4)"),
            (6, 11, "real*8", "real(8)"),
            (17, 8, "real*4", "real(4)"),
            (18, 12, "complex*8", "complex(8)"),
        ]
        .iter()
        .map(|(line, col, from, to)| {
            let msg = format!("{from} is non-standard, use {to}");
            violation!(&msg, *line, *col)
        })
        .collect();

        let rule = StarKind::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
