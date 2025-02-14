// Adapated from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

/// Fake rules for testing fortitude's behaviour
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_text_size::{TextRange, TextSize};

use crate::rules::Rule;

pub(crate) const TEST_RULES: &[Rule] = &[
    Rule::StableTestRule,
    Rule::StableTestRuleSafeFix,
    Rule::StableTestRuleUnsafeFix,
    Rule::StableTestRuleDisplayOnlyFix,
    Rule::PreviewTestRule,
    Rule::DeprecatedTestRule,
    Rule::AnotherDeprecatedTestRule,
    Rule::RemovedTestRule,
    Rule::AnotherRemovedTestRule,
    Rule::RedirectedFromTestRule,
    Rule::RedirectedToTestRule,
    Rule::RedirectedFromPrefixTestRule,
];

pub(crate) trait TestRule {
    fn check() -> Option<Diagnostic>;
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct StableTestRule;

impl Violation for StableTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a stable test rule.".to_string()
    }
}

impl TestRule for StableTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct StableTestRuleSafeFix;

impl Violation for StableTestRuleSafeFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a stable test rule with a safe fix.".to_string()
    }
}

impl TestRule for StableTestRuleSafeFix {
    fn check() -> Option<Diagnostic> {
        let comment = "! fix from stable-test-rule-safe-fix\n".to_string();

        Some(
            Diagnostic::new(Self {}, TextRange::default())
                .with_fix(Fix::safe_edit(Edit::insertion(comment, TextSize::new(0)))),
        )
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct StableTestRuleUnsafeFix;

impl Violation for StableTestRuleUnsafeFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a stable test rule with an unsafe fix.".to_string()
    }
}

impl TestRule for StableTestRuleUnsafeFix {
    fn check() -> Option<Diagnostic> {
        let comment = "! fix from stable-test-rule-unsafe-fix\n".to_string();
        Some(
            Diagnostic::new(Self {}, TextRange::default())
                .with_fix(Fix::unsafe_edit(Edit::insertion(comment, TextSize::new(0)))),
        )
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct StableTestRuleDisplayOnlyFix;

impl Violation for StableTestRuleDisplayOnlyFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a stable test rule with a display only fix.".to_string()
    }
}

impl TestRule for StableTestRuleDisplayOnlyFix {
    fn check() -> Option<Diagnostic> {
        let comment = "! fix from stable-test-rule-display-only-fix\n".to_string();
        Some(
            Diagnostic::new(Self {}, TextRange::default()).with_fix(Fix::display_only_edit(
                Edit::insertion(comment, TextSize::new(0)),
            )),
        )
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct PreviewTestRule;

impl Violation for PreviewTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a preview test rule.".to_string()
    }
}

impl TestRule for PreviewTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedTestRule;

impl Violation for DeprecatedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a deprecated test rule.".to_string()
    }
}

impl TestRule for DeprecatedTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct AnotherDeprecatedTestRule;

impl Violation for AnotherDeprecatedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is another deprecated test rule.".to_string()
    }
}

impl TestRule for AnotherDeprecatedTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct RemovedTestRule;

impl Violation for RemovedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a removed test rule.".to_string()
    }
}

impl TestRule for RemovedTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct AnotherRemovedTestRule;

impl Violation for AnotherRemovedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a another removed test rule.".to_string()
    }
}

impl TestRule for AnotherRemovedTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct RedirectedFromTestRule;

impl Violation for RedirectedFromTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a test rule that was redirected to another.".to_string()
    }
}

impl TestRule for RedirectedFromTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct RedirectedToTestRule;

impl Violation for RedirectedToTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a test rule that was redirected from another.".to_string()
    }
}

impl TestRule for RedirectedToTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}

/// ## What it does
/// Fake rule for testing.
///
/// ## Why is this bad?
/// Tests must pass!
///
/// ## Example
/// ```f90
/// foo
/// ```
///
/// Use instead:
/// ```f90
/// bar
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct RedirectedFromPrefixTestRule;

impl Violation for RedirectedFromPrefixTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Hey this is a test rule that was redirected to another by prefix.".to_string()
    }
}

impl TestRule for RedirectedFromPrefixTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}
