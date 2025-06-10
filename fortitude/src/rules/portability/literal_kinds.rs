use crate::ast::{dtype_is_plain_number, FortitudeNode};
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use lazy_regex::regex_is_match;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

fn iso_fortran_env_param<S: AsRef<str>>(dtype: S, literal: u8) -> Option<String> {
    match dtype.as_ref() {
        "integer" | "logical" => {
            if matches!(literal, 1u8 | 2u8 | 4u8 | 8u8) {
                Some(format!("int{}", literal * 8))
            } else {
                None
            }
        }
        "real" | "complex" => {
            if matches!(literal, 4u8 | 8u8 | 16u8) {
                Some(format!("real{}", literal * 8))
            } else {
                None
            }
        }
        _ => None,
    }
}

// TODO rules for intrinsic kinds in real(x, [KIND]) and similar type casting functions

/// ## What it does
/// Checks for use of raw number literals as kinds
///
/// ## Why is this bad?
/// Rather than setting an intrinsic type's kind using an integer literal, such as
/// `real(8)` or `integer(kind=4)`, consider setting kinds using parameters in the
/// intrinsic module `iso_fortran_env` such as `real64` and `int32`. For
/// C-compatible types, consider instead `iso_c_binding` types such as
/// `real(c_double)`.
///
/// Although it is widely believed that `real(8)` represents an 8-byte floating
/// point (and indeed, this is the case for most compilers and architectures),
/// there is nothing in the standard to mandate this, and compiler vendors are free
/// to choose any mapping between kind numbers and machine precision. This may lead
/// to surprising results if your code is ported to another machine or compiler.
///
/// For floating point variables, we recommended using `real(sp)` (single
/// precision), `real(dp)` (double precision), and `real(qp)` (quadruple precision),
/// using:
///
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: sp => real32, &
///                                          dp => real64, &
///                                          qp => real128
/// ```
///
/// Or alternatively:
///
/// ```f90
/// integer, parameter :: sp = selected_real_kind(6, 37)
/// integer, parameter :: dp = selected_real_kind(15, 307)
/// integer, parameter :: qp = selected_real_kind(33, 4931)
/// ```
///
/// Some prefer to set one precision parameter `wp` (working precision), which is
/// set in one module and used throughout a project.
///
/// Integer sizes may be set similarly:
///
/// ```f90
/// integer, parameter :: i1 = selected_int_kind(2)  ! 8 bits
/// integer, parameter :: i2 = selected_int_kind(4)  ! 16 bits
/// integer, parameter :: i4 = selected_int_kind(9)  ! 32 bits
/// integer, parameter :: i8 = selected_int_kind(18) ! 64 bits
/// ```
///
/// Or:
///
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: i1 => int8, &
///                                          i2 => int16, &
///                                          i4 => int32, &
///                                          i8 => int64
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct LiteralKind {
    dtype: String,
    literal: u8,
}

impl Violation for LiteralKind {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { dtype, literal } = self;
        format!("{dtype} kind set with number literal '{literal}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { dtype, literal } = self;
        iso_fortran_env_param(dtype, *literal)
            .map(|param| format!("Use the parameter '{param}' from 'iso_fortran_env'"))
    }
}

impl AstRule for LiteralKind {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        let dtype = node.child(0)?.to_text(src)?.to_lowercase();
        // TODO: Deal with characters
        if !dtype_is_plain_number(dtype.as_str()) {
            return None;
        }

        let kind_node = node.child_by_field_name("kind")?;
        let literal_node = integer_literal_kind(&kind_node, src)?;
        let literal: u8 = literal_node.to_text(src)?.parse().ok()?;
        some_vec![Diagnostic::from_node(
            Self { dtype, literal },
            &literal_node
        )]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

/// Return any kind spec that is a number literal
fn integer_literal_kind<'a>(node: &'a Node, src: &str) -> Option<Node<'a>> {
    if let Some(literal) = node.child_with_name("number_literal") {
        return Some(literal);
    }
    if let Some(literal) = node
        .kwarg_value("kind", src)
        .filter(|v| v.kind() == "number_literal")
    {
        return Some(literal);
    }
    None
}

/// ## What it does
/// Checks for using an integer literal as a kind suffix
///
/// ## Why is this bad?
/// Using an integer literal as a kind specifier gives no guarantees regarding the
/// precision of the type, as kind numbers are not specified in the Fortran
/// standards. It is recommended to use parameter types from `iso_fortran_env`:
///
/// ```f90
/// use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
/// ```
///
/// or alternatively:
///
/// ```f90
/// integer, parameter :: sp => selected_real_kind(6, 37)
/// integer, parameter :: dp => selected_real_kind(15, 307)
/// ```
///
/// Floating point constants can then be specified as follows:
///
/// ```f90
/// real(sp), parameter :: sqrt2 = 1.41421_sp
/// real(dp), parameter :: pi = 3.14159265358979_dp
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct LiteralKindSuffix {
    literal: String,
    suffix: u8,
}

impl Violation for LiteralKindSuffix {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { literal, suffix } = self;
        format!("'{literal}' has literal kind suffix '{suffix}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { literal, suffix } = self;
        let dtype = if regex_is_match!(r"^(\d*\.\d*|\d*\.*\d*[eE]-?\d+)", literal) {
            "real"
        } else {
            "integer"
        };
        iso_fortran_env_param(dtype, *suffix)
            .map(|param| format!("Use the parameter '{param}' from 'iso_fortran_env'"))
    }
}

impl AstRule for LiteralKindSuffix {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        let kind = node.child_by_field_name("kind")?;
        if kind.kind() != "number_literal" {
            return None;
        }
        let literal = node.to_text(src)?.to_string();
        let suffix: u8 = kind.to_text(src)?.parse().ok()?;
        some_vec![Diagnostic::from_node(Self { literal, suffix }, &kind)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["number_literal"]
    }
}
