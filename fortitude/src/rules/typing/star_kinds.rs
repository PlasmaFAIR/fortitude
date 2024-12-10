use crate::ast::{dtype_is_plain_number, strip_line_breaks, FortitudeNode};
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
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
/// Fixes to this rule are considered unsafe, as while `dtype*N` is generally
/// understood to mean a `dtype` that occupied `N` bytes, this does not necessarily
/// correspond to `dtype(N)`, which is a `dtype` of 'kind' `N`. For example, the NAG
/// compiler may be conigured to use a sequential kind system in which `real*8`
/// corresponds to `real(2)`
///
/// In a future version, we hope to upgrade this to a safe fix by use of parameters
/// in `iso_fortran_env`, as `real*8` should always correspond to `real(real64)`.
#[violation]
pub struct StarKind {
    dtype: String,
    size: String,
    kind: String,
}

impl Violation for StarKind {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { dtype, size, .. } = self;
        format!("'{dtype}{size}' uses non-standard syntax")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { dtype, kind, .. } = self;
        Some(format!("Replace with '{dtype}({kind})'"))
    }
}

impl AstRule for StarKind {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
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
        let replacement = format!("{dtype}({kind})");
        let fix = Fix::unsafe_edit(node.edit_replacement(replacement));
        some_vec![Diagnostic::from_node(Self { dtype, size, kind }, &kind_node).with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
