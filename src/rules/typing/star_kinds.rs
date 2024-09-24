use crate::ast::{
    child_with_name, dtype_is_plain_number, parse_intrinsic_type, strip_line_breaks, to_text,
};
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use lazy_regex::regex_captures;
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
    fn check(&self, node: &Node, src: &str) -> Option<Violation> {
        let dtype = parse_intrinsic_type(node)?;
        if dtype_is_plain_number(dtype.as_str()) {
            if let Some(child) = child_with_name(node, "size") {
                let size = strip_line_breaks(to_text(&child, src)?);
                // Match anything beginning with a '*' followed by any amount of
                // whitespace and some digits. Parameters like real64 aren't
                // allowed in this syntax, so we don't need to worry about them.
                if let Some((_, kind)) = regex_captures!(r"^\*\s*(\d+)", size.as_str()) {
                    let msg = format!(
                        "{}{} is non-standard, use {}({})",
                        dtype,
                        size.replace(" ", "").replace("\t", ""),
                        dtype,
                        kind,
                    );
                    return Some(Violation::from_node(&msg, node));
                }
            }
        }
        None
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["variable_declaration", "function_statement"]
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

              if (x) then
                add_if = x + y
              else
                add_if = x
              end if
            end function

            subroutine complex_mul(x, y)
              real * 4, intent(in) :: x
              complex  *  8, intent(inout) :: y
              y = y * x
            end subroutine
            ",
        );

        let expected: Vec<Violation> = [
            (2, 1, "integer*8", "integer(8)"),
            (4, 3, "integer*4", "integer(4)"),
            (5, 3, "logical*4", "logical(4)"),
            (6, 3, "real*8", "real(8)"),
            (17, 3, "real*4", "real(4)"),
            (18, 3, "complex*8", "complex(8)"),
        ]
        .iter()
        .map(|(line, col, from, to)| {
            let msg = format!("{} is non-standard, use {}", from, to);
            violation!(&msg, *line, *col)
        })
        .collect();

        let rule = StarKind::new(&default_settings());
        let actual = rule.apply(source.as_str())?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
