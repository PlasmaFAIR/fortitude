use crate::ast::{dtype_is_plain_number, FortitudeNode};
use crate::settings::Settings;
use crate::{ASTRule, Rule, Violation};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// Defines rules that discourage the use of raw number literals as kinds, as this can result in
/// non-portable code.

// TODO rules for intrinsic kinds in real(x, [KIND]) and similar type casting functions

pub struct LiteralKind {}

impl Rule for LiteralKind {
    fn new(_settings: &Settings) -> Self {
        LiteralKind {}
    }

    fn explain(&self) -> &'static str {
        "
        Rather than setting an intrinsic type's kind using an integer literal, such as
        `real(8)` or `integer(kind=4)`, consider setting kinds using parameters in the
        intrinsic module `iso_fortran_env` such as `real64` and `int32`. For
        C-compatible types, consider instead `iso_c_binding` types such as
        `real(c_double)`.

        Although it is widely believed that `real(8)` represents an 8-byte floating
        point (and indeed, this is the case for most compilers and architectures),
        there is nothing in the standard to mandate this, and compiler vendors are free
        to choose any mapping between kind numbers and machine precision. This may lead
        to surprising results if your code is ported to another machine or compiler.

        For floating point variables, we recommended using `real(sp)` (single
        precision), `real(dp)` (double precision), and `real(qp)` (quadruple precision),
        using:

        ```
        use, intrinsic :: iso_fortran_env, only: sp => real32, &
                                                 dp => real64, &
                                                 qp => real128
        ```

        Or alternatively:

        ```
        integer, parameter :: sp = selected_real_kind(6, 37)
        integer, parameter :: dp = selected_real_kind(15, 307)
        integer, parameter :: qp = selected_real_kind(33, 4931)
        ```

        Some prefer to set one precision parameter `wp` (working precision), which is
        set in one module and used throughout a project.

        Integer sizes may be set similarly:

        ```
        integer, parameter :: i1 = selected_int_kind(2)  ! 8 bits
        integer, parameter :: i2 = selected_int_kind(4)  ! 16 bits
        integer, parameter :: i4 = selected_int_kind(9)  ! 32 bits
        integer, parameter :: i8 = selected_int_kind(18) ! 64 bits
        ```

        Or:

        ```
        use, intrinsic :: iso_fortran_env, only: i1 => int8, &
                                                 i2 => int16, &
                                                 i4 => int32, &
                                                 i8 => int64
        ```
        "
    }
}

impl ASTRule for LiteralKind {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let src = src.source_text();
        let dtype = node.child(0)?.to_text(src)?.to_lowercase();
        // TODO: Deal with characters
        if !dtype_is_plain_number(dtype.as_str()) {
            return None;
        }

        let kind_node = node.child_by_field_name("kind")?;
        let literal_value = integer_literal_kind(&kind_node, src)?;
        // TODO: Can we recommend the "correct" size? Although
        // non-standard, `real*8` _usually_ means `real(real64)`
        let msg = format!(
            "{dtype} kind set with number literal '{}', use 'iso_fortran_env' parameter",
            literal_value.to_text(src)?
        );
        some_vec![Violation::from_node(msg, &literal_value)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}

/// Return any kind spec that is a number literal
fn integer_literal_kind<'a>(node: &'a Node, src: &str) -> Option<Node<'a>> {
    if let Some(literal) = node.child_with_name("number_literal") {
        return Some(literal);
    }

    for child in node.named_children(&mut node.walk()) {
        if child.kind() == "number_literal" {
            return Some(child);
        }

        if child.kind() != "keyword_argument" {
            continue;
        }

        // find instances of `kind=8` etc
        let name = child.child_by_field_name("name")?;
        if &name.to_text(src)?.to_lowercase() != "kind" {
            continue;
        }
        let value = child.child_by_field_name("value")?;
        if value.kind() == "number_literal" {
            return Some(value);
        }
    }
    None
}

pub struct LiteralKindSuffix {}

impl Rule for LiteralKindSuffix {
    fn new(_settings: &Settings) -> Self {
        LiteralKindSuffix {}
    }

    fn explain(&self) -> &'static str {
        "
        Using an integer literal as a kind specifier gives no guarantees regarding the
        precision of the type, as kind numbers are not specified in the Fortran
        standards. It is recommended to use parameter types from `iso_fortran_env`:

        ```
        use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
        ```

        or alternatively:

        ```
        integer, parameter :: sp => selected_real_kind(6, 37)
        integer, parameter :: dp => selected_real_kind(15, 307)
        ```

        Floating point constants can then be specified as follows:

        ```
        real(sp), parameter :: sqrt2 = 1.41421_sp
        real(dp), parameter :: pi = 3.14159265358979_dp
        ```
        "
    }
}

impl ASTRule for LiteralKindSuffix {
    fn check(&self, node: &Node, src: &SourceFile) -> Option<Vec<Violation>> {
        let src = src.source_text();
        let kind = node.child_by_field_name("kind")?;
        if kind.kind() != "number_literal" {
            return None;
        }
        let msg = format!(
            "'{}' has literal suffix '{}', use 'iso_fortran_env' parameter",
            node.to_text(src)?,
            &kind.to_text(src)?,
        );
        some_vec![Violation::from_node(msg, &kind)]
    }

    fn entrypoints(&self) -> Vec<&'static str> {
        vec!["number_literal"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::default_settings, test_file};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_literal_kind() -> anyhow::Result<()> {
        let source = test_file(
            "
            integer(8) function add_if(x, y, z)
              integer :: w
              integer(kind=2), intent(in) :: x
              integer(i32), intent(in) :: y
              logical(kind=4), intent(in) :: z

              if (x) then
                add_if = x + y
              else
                add_if = x
              end if
            end function

            subroutine complex_mul(x, y)
              real(8), intent(in) :: x
              complex(4), intent(inout) :: y
              real :: z = 0.5
              y = y * x
            end subroutine

            complex(real64) function complex_add(x, y)
              real(real64), intent(in) :: x
              complex(kind=4), intent(in) :: y
              complex_add = y + x
            end function
            ",
        );
        let expected: Vec<Violation> = [
            (1, 8, 1, 9, "integer", 8),
            (3, 15, 3, 16, "integer", 2),
            (5, 15, 5, 16, "logical", 4),
            (15, 7, 15, 8, "real", 8),
            (16, 10, 16, 11, "complex", 4),
            (23, 15, 23, 16, "complex", 4),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, kind, literal)| {
            Violation::from_start_end_line_col(
                format!("{kind} kind set with number literal '{literal}', use 'iso_fortran_env' parameter"),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = LiteralKind::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_literal_kind_suffix() -> anyhow::Result<()> {
        let source = test_file(
            "
            use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

            real(sp), parameter :: x1 = 1.234567_4
            real(dp), parameter :: x2 = 1.234567_dp
            real(dp), parameter :: x3 = 1.789d3
            real(dp), parameter :: x4 = 9.876_8
            real(sp), parameter :: x5 = 2.468_sp
            ",
        );
        let expected: Vec<Violation> = [
            (3, 37, 3, 38, "1.234567_4", "4"),
            (6, 34, 6, 35, "9.876_8", "8"),
        ]
        .iter()
        .map(|(start_line, start_col, end_line, end_col, num, kind)| {
            Violation::from_start_end_line_col(
                format!("'{num}' has literal suffix '{kind}', use 'iso_fortran_env' parameter",),
                &source,
                *start_line,
                *start_col,
                *end_line,
                *end_col,
            )
        })
        .collect();
        let rule = LiteralKindSuffix::new(&default_settings());
        let actual = rule.apply(&source)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
