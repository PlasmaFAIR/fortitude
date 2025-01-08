use ruff_diagnostics::Violation;
use ruff_macros::{derive_message_formats, violation};

/// ## What it does
/// Checks for invalid rules in allow comments.
///
/// ## Why is this bad?
/// Invalid rules in allow comments are likely typos or mistakes.
///
/// ## Example
/// The user meant `implicit-typing` but made a mistake:
/// ```f90
/// ! allow(implicit-typos)
/// program test
/// end program test
/// ```
#[violation]
pub struct InvalidRuleCodeOrName {
    pub message: String,
}

/// E011
impl Violation for InvalidRuleCodeOrName {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { message } = self;
        format!("{message}")
    }
}
