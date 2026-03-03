use crate::ast::FortitudeNode;
use crate::settings::{CheckSettings, FortranStandard};
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

// TODO Check that 'used' entity is actually used somewhere

/// ## What it does
/// Checks whether `use` statements are used correctly.
///
/// ## Why is this bad?
/// When using a module, it is recommended to add an 'only' clause to specify which
/// components you intend to use:
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use, intrinsic :: iso_fortran_env
///
/// ! Better
/// use, intrinsic :: iso_fortran_env, only: int32, real64
/// ```
///
/// This makes it easier for programmers to understand where the symbols in your
/// code have come from, and avoids introducing many unneeded components to your
/// local scope.
///
/// ## Options
/// - `check.use-statements.allow-bare-use`
#[derive(ViolationMetadata)]
pub(crate) struct UseAll {}

impl Violation for UseAll {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'use' statement missing 'only' clause".to_string()
    }
}

impl AstRule for UseAll {
    fn check(
        settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let module_name = node
            .child_with_name("module_name")?
            .to_text(src.source_text())?
            .to_lowercase();

        if !settings
            .use_statements
            .allow_bare_use
            .contains(&module_name)
            && node.child_with_name("included_items").is_none()
        {
            return some_vec![Diagnostic::from_node(UseAll {}, node)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["use_statement"]
    }
}

/// ## What it does
/// Checks whether `use` statements for intrinsic modules specify `intrinsic` or
/// `non_intrinsic`.
///
/// ## Why is this bad?
/// The compiler will default to using a non-intrinsic module, if there is one,
/// so not specifying the `intrinsic` modifier on intrinsic modules may lead to
/// the compiler version being shadowed by a different module with the same name.
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use :: iso_fortran_env, only: int32, real64
///
/// ! Better
/// use, intrinsic :: iso_fortran_env, only: int32, real64
/// ```
///
/// This ensures the compiler will use the built-in module instead of a different
/// module with the same name.
///
/// This feature is only available in Fortran 2003 and later.
#[derive(ViolationMetadata)]
pub(crate) struct MissingIntrinsic {}

const INTRINSIC_MODULES: &[&str] = &[
    "iso_fortran_env",
    "iso_c_binding",
    "ieee_exceptions",
    "ieee_arithmetic",
    "ieee_features",
];

impl Violation for MissingIntrinsic {
    #[derive_message_formats]
    fn message(&self) -> String {
        "'use' for intrinsic module missing 'intrinsic' modifier".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'intrinsic'".to_string())
    }
}

impl AstRule for MissingIntrinsic {
    fn check(
        settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // Feature only available in Fortran 2003 and later
        if settings.target_std < FortranStandard::F2003 {
            return None;
        }
        let module_name = node
            .child_with_name("module_name")?
            .to_text(src.source_text())?
            .to_lowercase();

        if INTRINSIC_MODULES.iter().any(|&m| m == module_name)
            && node
                .children(&mut node.walk())
                .filter_map(|child| child.to_text(src.source_text()))
                .all(|child| child != "intrinsic" && child != "non_intrinsic")
        {
            let intrinsic = if node.child(1)?.kind() == "::" {
                ", intrinsic"
            } else {
                ", intrinsic ::"
            };

            let use_field = node
                .children(&mut node.walk())
                .find(|&child| child.to_text(src.source_text()) == Some("use"))?;
            let start_pos = use_field.end_textsize();
            let fix = Fix::unsafe_edit(Edit::insertion(intrinsic.to_string(), start_pos));

            return some_vec![Diagnostic::from_node(MissingIntrinsic {}, node).with_fix(fix)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["use_statement"]
    }
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub allow_bare_use: Vec<String>,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.use_statements",
                fields = [self.allow_bare_use | debug]
            }
            Ok(())
        }
    }
}
