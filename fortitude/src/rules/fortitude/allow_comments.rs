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
///
/// ## Why is this bad?
/// When one of Fortitude's rule codes has been redirected, the implication is that the rule has
/// been deprecated in favor of another rule or code. To keep your codebase
/// consistent and up-to-date, prefer the canonical rule code over the deprecated
/// code.
///
/// ## Example
/// ```f90
/// ! allow(T001)
/// program foo
/// ```
///
/// Use instead:
/// ```f90
/// ! allow(implicit-typing)
/// program foo
/// ```
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
/// Checks for `allow` comments that aren't applicable.
///
/// ## Why is this bad?
/// An `allow` comment that no longer matches any diagnostic violations
/// is likely included by mistake, and should be removed to avoid confusion.
///
/// ## Example
/// ```f90
/// ! allow(implicit-typing)
/// program foo
///   implicit none
/// ```
///
/// Use instead:
/// ```f90
/// program foo
///   implicit none
/// ```
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
/// Checks for `allow` comments with duplicated rules.
///
/// ## Why is this bad?
/// Duplicated rules in `allow` comments are very likely to be mistakes, and
/// should be removed to avoid confusion.
///
/// ## Example
/// ```f90
/// ! allow(C001, C002, C001)
/// program foo
/// ```
///
/// Use instead:
/// ```f90
/// ! allow(C001, C002)
/// program foo
/// ```
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
/// Checks for `allow` comments that are disabled globally.
///
/// ## Why is this bad?
/// These `allow` comments will have no effect, and should be removed to avoid
/// confusion.
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
