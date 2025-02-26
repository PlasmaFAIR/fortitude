use ruff_diagnostics::{AlwaysFixableViolation, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};

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
#[derive(ViolationMetadata)]
pub(crate) struct InvalidRuleCodeOrName {
    pub rule: String,
}

/// FORT001
impl Violation for InvalidRuleCodeOrName {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { rule } = self;
        format!("Unknown rule or code `{rule}` in allow comment")
    }
}

/// ## What it does
/// Checks for redirected rules in allow comments.
#[derive(ViolationMetadata)]
pub(crate) struct RedirectedAllowComment {
    pub original: String,
    pub new_code: String,
    pub new_name: String,
}

impl AlwaysFixableViolation for RedirectedAllowComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            original, new_code, ..
        } = self;
        format!("`{original}` is a redirect to `{new_code}`")
    }

    fn fix_title(&self) -> String {
        let Self {
            new_code, new_name, ..
        } = self;
        format!("Replace with `{new_code}` or `{new_name}`")
    }
}

/// ## What it does
/// Checks for allow comments that aren't applicable.
///
/// ## Why is this bad?
/// Probably a mistake
#[derive(ViolationMetadata)]
pub(crate) struct UnusedAllowComment {
    pub rule: String,
}

impl AlwaysFixableViolation for UnusedAllowComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { rule } = self;
        format!("Unused rule `{rule}` in allow comment")
    }

    fn fix_title(&self) -> String {
        "Remove unused allow comment".to_string()
    }
}

/// ## What it does
/// Checks for allow comments that aren't applicable.
///
/// ## Why is this bad?
/// Probably a mistake
#[derive(ViolationMetadata)]
pub(crate) struct DuplicatedAllowComment {
    pub rule: String,
}

impl AlwaysFixableViolation for DuplicatedAllowComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { rule } = self;
        format!("Duplicated rule `{rule}` in allow comment")
    }

    fn fix_title(&self) -> String {
        "Remove duplicated allow comment".to_string()
    }
}

/// ## What it does
/// Checks for allow comments that aren't applicable.
///
/// ## Why is this bad?
/// Probably a mistake
#[derive(ViolationMetadata)]
pub(crate) struct DisabledAllowComment {
    pub rule: String,
}

impl AlwaysFixableViolation for DisabledAllowComment {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { rule } = self;
        format!("Disabled rule `{rule}` in allow comment")
    }

    fn fix_title(&self) -> String {
        "Remove disabled allow comment".to_string()
    }
}
