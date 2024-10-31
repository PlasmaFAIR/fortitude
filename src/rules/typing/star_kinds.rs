use crate::ast::{dtype_is_plain_number, strip_line_breaks, FortitudeNode};
use crate::settings::Settings;
use crate::{ASTRule, FromTSNode, Rule};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for non-standard kind specifiers such as `int*4` or `real*8`
///
/// ## Why is this bad?
/// Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
/// avoided. For these cases, consider instead using 'real(real64)' or
/// 'integer(int32)', where 'real64' and 'int32' may be found in the intrinsic
/// module 'iso_fortran_env'. You may also wish to determine kinds using the
/// built-in functions 'selected_real_kind' and 'selected_int_kind'.
///
/// Also prefers the use of `character(len=*)` to
/// `character*(*)`, as although the latter is permitted by the standard, the former is
/// more explicit.
#[violation]
pub struct StarKind {
    dtype: String,
    size: String,
    kind: String,
}

impl Violation for StarKind {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { dtype, size, kind } = self;
        format!("{dtype}{size} is non-standard, use {dtype}({kind})")
    }
}

impl Rule for StarKind {
    fn new(_settings: &Settings) -> Self {
        StarKind {
            dtype: String::default(),
            size: String::default(),
            kind: String::default(),
        }
    }
}

impl ASTRule for StarKind {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
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
        let kind = literal.to_text(src)?.to_string();
        // TODO: Better suggestion, rather than use integer literal
        some_vec![Diagnostic::from_node(
            Self { dtype, size, kind },
            &kind_node
        )]
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
    fn test_star_kind() -> anyhow::Result<()> {
        let source = test_file(
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

        let expected: Vec<_> = [
            (1, 7, 1, 9, "integer", "*8", "8"),
            (3, 10, 3, 12, "integer", "*4", "4"),
            (4, 9, 4, 14, "logical", "*4", "4"),
            (5, 10, 6, 4, "real", "*8", "8"),
            (16, 7, 16, 10, "real", "*4", "4"),
            (17, 11, 17, 15, "complex", "*8", "8"),
        ]
        .iter()
        .map(
            |(start_line, start_col, end_line, end_col, dtype, size, kind)| {
                Diagnostic::from_start_end_line_col(
                    StarKind {
                        dtype: dtype.to_string(),
                        size: size.to_string(),
                        kind: kind.to_string(),
                    },
                    &source,
                    *start_line,
                    *start_col,
                    *end_line,
                    *end_col,
                )
            },
        )
        .collect();

        let rule = StarKind::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);

        Ok(())
    }
}
