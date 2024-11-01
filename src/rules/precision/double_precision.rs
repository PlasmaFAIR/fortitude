use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{ASTRule, FromASTNode, Rule};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

// TODO rule to prefer 1.23e4_sp over 1.23e4, and 1.23e4_dp over 1.23d4

/// ## What it does
/// Checks for use of 'double precision' and 'double complex' types.
///
/// ## Why is this bad?
/// The 'double precision' type does not guarantee a 64-bit floating point number
/// as one might expect. It is instead required to be twice the size of a default
/// 'real', which may vary depending on your system and can be modified by compiler
/// arguments. For portability, it is recommended to use `real(dp)`, with `dp` set
/// in one of the following ways:
///
/// - `use, intrinsic :: iso_fortran_env, only: dp => real64`
/// - `integer, parameter :: dp = selected_real_kind(15, 307)`
///
/// For code that should be compatible with C, you should instead use
/// `real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
#[violation]
pub struct DoublePrecision {
    original: String, // TODO: could be &'static str
    preferred: String,
}

impl DoublePrecision {
    fn try_new<S: AsRef<str>>(original: S) -> Option<Self> {
        match original.as_ref() {
            "double precision" => Some(Self {
                original: original.as_ref().to_string(),
                preferred: "real(real64)".to_string(),
            }),
            "double complex" => Some(Self {
                original: original.as_ref().to_string(),
                preferred: "complex(real64)".to_string(),
            }),
            _ => None,
        }
    }
}

impl Violation for DoublePrecision {
    #[derive_message_formats]
    fn message(&self) -> String {
        let DoublePrecision {
            original,
            preferred,
        } = self;
        format!("prefer '{preferred}' to '{original}' (see 'iso_fortran_env')")
    }
}

impl Rule for DoublePrecision {
    fn new(_settings: &Settings) -> Self {
        Self {
            original: String::default(),
            preferred: String::default(),
        }
    }
}

impl ASTRule for DoublePrecision {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let txt = node.to_text(src.source_text())?.to_lowercase();
        some_vec![Diagnostic::from_node(DoublePrecision::try_new(txt)?, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_file, FromStartEndLineCol};
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
        let expected: Vec<_> = [
            (1, 0, 1, 16, "double precision"),
            (2, 2, 2, 18, "double precision"),
            (7, 2, 7, 18, "double precision"),
            (12, 2, 12, 18, "double precision"),
            (13, 2, 13, 16, "double complex"),
            (14, 2, 14, 16, "double complex"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, kind)| {
            let msg = DoublePrecision::try_new(kind).unwrap();
            Diagnostic::from_start_end_line_col(
                msg,
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let actual = DoublePrecision::apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
