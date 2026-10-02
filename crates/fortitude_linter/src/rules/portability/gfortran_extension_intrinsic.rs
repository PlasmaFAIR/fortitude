use crate::diagnostics::{Diagnostic, Violation};
use crate::rules::utilities::is_assignment_lhs;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use fortitude_sitter::Node;
use ruff_macros::derive_message_formats;

/// ## What does it do?
/// Checks for gfortran random number extensions, `srand` and `rand`, and suggests
/// to replace them with the standard intrinsics `random_seed` and `random_number`
/// respectively.
///
/// ## Why is this bad?
/// Using compiler specific instrinsics is inherently unportable as other users
/// will need to compile their code with the specific compiler. It can also make
/// code harder to understand if someone is not familiar with the extension. Using
/// Fortran standard implementations is preferred, and in some cases implement improved
/// algorithms.
///
/// ## Example
/// ```f90
/// call srand(seed)
/// x = rand(seed)
/// ```
///
/// ## Use instead
/// ```f90
/// integer, allocatable :: seed(:)
/// integer :: n
///
/// call random_seed(size = n)
/// allocate(seed(n))
/// seed(1) = 12345
/// call random_seed(put=seed)
/// call random_number(x)
/// ```
///
/// Prefer using the Fortran standard intrinsics `random_seed` and `random_number` instead.
/// These are not interchangeable replacements, as for example, `random_number(x)` is a subroutine
/// that modifies `x`, whereas `rand()` is a function that returns a single `real`.
///
/// ## References
/// - [GFortran docs for `srand`](https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html)
/// - [GFortran docs for `rand`](https://gcc.gnu.org/onlinedocs/gfortran/RAND.html)
/// - [GFortran docs for `random_seed`](https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html)
/// - [GFortran docs for `random_number`](https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html)

#[derive(ViolationMetadata)]
pub(crate) struct GfortranRandomExtension {
    func: String,
    kind: GfortranExtensionKind,
}

impl Violation for GfortranRandomExtension {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { func, .. } = self;
        format!("possible gfortran extension function '{func}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { kind, .. } = self;
        match kind {
            GfortranExtensionKind::Srand => Some(format!(
                "Use Fortran standard intrinsic `random_seed` instead. See docs for examples."
            )),
            GfortranExtensionKind::Rand => Some(format!(
                "Use Fortran standard intrinsic `random_number` instead. See docs for examples."
            )),
        }
    }
}

enum GfortranExtensionKind {
    Srand,
    Rand,
}

fn match_extension_kind(name: &str) -> Option<GfortranExtensionKind> {
    match name.to_uppercase().as_str() {
        "SRAND" => Some(GfortranExtensionKind::Srand),
        "RAND" => Some(GfortranExtensionKind::Rand),
        _ => None,
    }
}

fn is_array_access(node: &Node) -> bool {
    let child_node = node.child_with_name("argument_list");
    match child_node {
        Some(node) => node.child_with_name("extent_specifier").is_some(),
        None => false,
    }
}

impl AstRule for GfortranRandomExtension {
    fn check<'a>(context: &'a CheckContext, node: &'a Node) -> Option<Vec<Diagnostic>> {
        // Check that this is not actually an array access/assignment
        if is_assignment_lhs(node) || is_array_access(node) {
            return None;
        }

        let name_node = node.child_with_name("identifier")?;
        let func = name_node.text().to_string();
        let kind = match_extension_kind(func.as_str())?;

        some_vec![context.create_diagnostic(Self { func, kind }, name_node)]
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["call_expression", "subroutine_call"]
    }
}
