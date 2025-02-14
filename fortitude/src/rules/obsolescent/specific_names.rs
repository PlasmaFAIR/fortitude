use crate::ast::FortitudeNode;
use crate::rules::utilities;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Fix, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

fn map_specific_intrinsic_functions(name: &str) -> Option<&'static str> {
    match name {
        // Real-specific functions
        "ALOG" => Some("LOG"),
        "ALOG10" => Some("LOG10"),
        "AMOD" => Some("MOD"),

        "AMAX1" => Some("MAX"),
        "AMIN1" => Some("MIN"),

        // Complex-specific functions
        "CABS" => Some("ABS"),
        "CCOS" => Some("COS"),
        "CEXP" => Some("EXP"),
        "CLOG" => Some("LOG"),
        "CSIN" => Some("SIN"),
        "CSQRT" => Some("SQRT"),

        // Double precision-specific functions
        "DABS" => Some("ABS"),
        "DACOS" => Some("ACOS"),
        "DASIN" => Some("ASIN"),
        "DATAN" => Some("ATAN"),
        "DATAN2" => Some("ATAN2"),
        "DCOS" => Some("COS"),
        "DCOSH" => Some("COSH"),
        "DDIM" => Some("DIM"),
        "DEXP" => Some("EXP"),
        "DINT" => Some("AINT"),
        "DLOG" => Some("LOG"),
        "DLOG10" => Some("LOG10"),
        "DMOD" => Some("MOD"),
        "DNINT" => Some("ANINT"),
        "DSIGN" => Some("SIGN"),
        "DSIN" => Some("SIN"),
        "DSINH" => Some("SINH"),
        "DSQRT" => Some("SQRT"),
        "DTAN" => Some("TAN"),
        "DTANH" => Some("TANH"),
        "IDNINT" => Some("NINT"),

        // Integer-specific functions
        "IABS" => Some("ABS"),
        "IDIM" => Some("DIM"),
        "ISIGN" => Some("SIGN"),
        _ => None,
    }
}

/// ## What does it do?
/// Checks for uses of the deprecated specific names of intrinsic functions.
///
/// ## Why is this bad?
/// Specific names of intrinsic functions can be obscure and hinder readability of
/// the code. Fortran 90 made these specific names redundant and recommends the use
/// of the generic names for calling intrinsic functions.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix B
///   'Obsolescent and Deleted Features'
#[derive(ViolationMetadata)]
pub(crate) struct SpecificName {
    func: String,
    new_func: String,
}

impl Violation for SpecificName {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { func, .. } = self;
        format!("deprecated type-specific function '{func}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { new_func, .. } = self;
        Some(format!("Use '{new_func}'"))
    }
}

impl AstRule for SpecificName {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let name_node = node.child_with_name("identifier")?;
        let func = name_node.to_text(src.source_text())?;

        let new_func = map_specific_intrinsic_functions(func.to_uppercase().as_str())?;
        let matched_case = utilities::match_original_case(func, new_func)?;

        let fix = Fix::unsafe_edit(name_node.edit_replacement(src, matched_case.clone()));
        some_vec![Diagnostic::from_node(
            Self {
                func: func.to_string(),
                new_func: matched_case
            },
            &name_node
        )
        .with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["call_expression"]
    }
}
