use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

// TODO rule to prefer 1.23e4_sp over 1.23e4, and 1.23e4_dp over 1.23d4

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
/// - [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/en/learn/best_practices/floating_point/)
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

impl AstRule for DoublePrecision {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let txt = node.to_text(src.source_text())?.to_lowercase();
        some_vec![Diagnostic::from_node(DoublePrecision::try_new(txt)?, node)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
