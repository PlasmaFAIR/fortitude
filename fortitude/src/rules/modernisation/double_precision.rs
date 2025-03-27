use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use lazy_regex::regex_captures;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for use of `double precision` and `double complex` types.
///
/// ## Why is this bad?
/// The `double precision` type does not guarantee a 64-bit floating point number
/// as one might expect, and instead is only required to have a higher decimal
/// precision than the default `real`, which may vary depending on your system
/// and can be modified by compiler arguments.
///
/// In modern Fortran, it is preferred to use `real` and `complex` and instead set
/// the required precision using 'kinds'. For portability, it is recommended to use
/// `real(dp)`, with `dp` set in one of the following ways:
///
/// - `use, intrinsic :: iso_fortran_env, only: dp => real64`
/// - `integer, parameter :: dp = selected_real_kind(15, 307)`
///
/// For code that should be compatible with C, you should instead use
/// `real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained: Incorporating Fortran
///   2018_, Oxford University Press, Appendix A 'Deprecated Features'
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)
#[derive(ViolationMetadata)]
pub(crate) struct DoublePrecision {
    original: String,
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
        let Self { original, .. } = self;
        format!("Use of '{original}' is discouraged")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { preferred, .. } = self;
        Some(format!("Prefer '{preferred}' (see 'iso_fortran_env')"))
    }
}

impl AstRule for DoublePrecision {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let txt = node.to_text(src.source_text())?.to_lowercase();
        some_vec![Diagnostic::from_node(DoublePrecision::try_new(txt)?, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

/// ## What it does
/// Checks for floating point literals that using `d` or `D` as the exponentiation.
///
/// ## Why is this bad?
/// Floating point literals using `d` or `D` for the exponent, such as `1.23d2`,
/// will be of the `double precision` kind. This is commonly assumed to be a
/// 64-bit float, but is not guaranteed to be so, and may vary depending on your
/// system and compiler arguments.
///
/// In modern Fortran, it is preferred to set the required precision using
/// 'kinds'. For portability, it is recommended to use `real(dp)`, with `dp` set
/// in one of the following ways:
///
/// - `use, intrinsic :: iso_fortran_env, only: dp => real64`
/// - `integer, parameter :: dp = selected_real_kind(15, 307)`
///
/// For code that should be compatible with C, you should instead use
/// `real(c_double)`, which may be found in the intrinsic module
/// `iso_c_binding`.  To ensure floating point literals match the kind of the
/// variable they are assigned to, it is recommended to use `e` or `E` for
/// exponentiation and a kind suffix, so `1.23d2` should be written as
/// `1.23e2_dp`.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained: Incorporating Fortran
///   2018_, Oxford University Press, Appendix A 'Deprecated Features'
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)
#[derive(ViolationMetadata)]
pub(crate) struct DoublePrecisionLiteral {
    original: String,
    preferred: String,
}

impl Violation for DoublePrecisionLiteral {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { original, .. } = self;
        format!("Use of 'd' exponentiation in '{original}' is discouraged")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { preferred, .. } = self;
        Some(format!("Prefer '{preferred}' (see 'iso_fortran_env')"))
    }
}

impl AstRule for DoublePrecisionLiteral {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let txt = node.to_text(src.source_text())?;
        if let Some((original, mantissa, exponent)) =
            regex_captures!(r"^(\d*\.*\d*)[dD](-?\d+)$", txt)
        {
            // Determine the immediate context in which we've found the literal.
            let mut parent = node.parent()?;
            while matches!(
                parent.kind(),
                "unary_expression" | "parenthesized_expression" | "complex_literal"
            ) {
                parent = parent.parent()?;
            }
            let grandparent = parent.parent()?;
            // Ok if being used in a kind statement or a type cast.
            // In the latter case, warnings should still be raised if precision would be
            // lost.
            // If it's the sole argument in a function call, the first parent must be
            // "argument_list", and the second must be "call_expression".
            if grandparent.kind() == "call_expression" {
                if let Some(identifier) = grandparent.child_with_name("identifier") {
                    let name = identifier.to_text(src.source_text())?.to_lowercase();
                    if name == "kind"
                        || matches!(name.as_str(), "real" | "cmplx" | "dbl" | "int" | "logical")
                    {
                        return None;
                    }
                }
            }

            let original = original.to_string();
            let preferred = format!("{mantissa}e{exponent}_real64");
            return some_vec![Diagnostic::from_node(
                DoublePrecisionLiteral {
                    original,
                    preferred
                },
                node
            )];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["number_literal"]
    }
}
