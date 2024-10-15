use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;
/// Defines rules to avoid the 'double precision' and 'double complex' types.

// TODO rule to prefer 1.23e4_sp over 1.23e4, and 1.23e4_dp over 1.23d4

fn double_precision_err_msg(dtype: &str) -> Option<String> {
    match dtype {
        "double precision" => Some(String::from(
            "prefer 'real(real64)' to 'double precision' (see 'iso_fortran_env')",
        )),
        "double complex" => Some(String::from(
            "prefer 'complex(real64)' to 'double complex' (see 'iso_fortran_env')",
        )),
        _ => None,
    }
}

pub struct DoublePrecision {}

impl Rule for DoublePrecision {
    fn new(_settings: &Settings) -> Self {
        Self {}
    }

    fn explain(&self) -> &'static str {
        "
        The 'double precision' type does not guarantee a 64-bit floating point number
        as one might expect. It is instead required to be twice the size of a default
        'real', which may vary depending on your system and can be modified by compiler
        arguments. For portability, it is recommended to use `real(dp)`, with `dp` set
        in one of the following ways:

        - `use, intrinsic :: iso_fortran_env, only: dp => real64`
        - `integer, parameter :: dp = selected_real_kind(15, 307)`

        For code that should be compatible with C, you should instead use
        `real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
        "
    }
}

impl ASTRule for DoublePrecision {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let txt = node.to_text(src.source_text())?.to_lowercase();
        if let Some(msg) = double_precision_err_msg(txt.as_str()) {
            return some_vec![Violation::from_node(msg.as_str(), node)];
        }
        None
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
    fn test_double_precision() -> anyhow::Result<()> {
        let source = test_file(
            "
            double precision function double(x)
              double precision, intent(in) :: x
              double = 2 * x
            end function

            subroutine triple(x)
              double precision, intent(inout) :: x
              x = 3 * x
            end subroutine

            function complex_mul(x, y)
              double precision, intent(in) :: x
              double complex, intent(in) :: y
              double complex :: complex_mul
              complex_mul = x * y
            end function
            ",
        );
        let expected: Vec<Violation> = [
            (1, 0, 1, 16, "double precision"),
            (2, 2, 2, 18, "double precision"),
            (7, 2, 7, 18, "double precision"),
            (12, 2, 12, 18, "double precision"),
            (13, 2, 13, 16, "double complex"),
            (14, 2, 14, 16, "double complex"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, kind)| {
            Violation::from_start_end_line_col(
                double_precision_err_msg(kind).unwrap(),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = DoublePrecision::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
