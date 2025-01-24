// Adapated from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

/// Fake rules for testing fortitude's behaviour
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
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
#[violation]
pub struct StableTestRule;

impl Violation for StableTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a stable test rule.")
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
#[violation]
pub struct StableTestRuleSafeFix;

impl Violation for StableTestRuleSafeFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a stable test rule with a safe fix.")
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
#[violation]
pub struct StableTestRuleUnsafeFix;

impl Violation for StableTestRuleUnsafeFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a stable test rule with an unsafe fix.")
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
#[violation]
pub struct StableTestRuleDisplayOnlyFix;

impl Violation for StableTestRuleDisplayOnlyFix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a stable test rule with a display only fix.")
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
#[violation]
pub struct PreviewTestRule;

impl Violation for PreviewTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a preview test rule.")
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
#[violation]
pub struct DeprecatedTestRule;

impl Violation for DeprecatedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a deprecated test rule.")
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
#[violation]
pub struct AnotherDeprecatedTestRule;

impl Violation for AnotherDeprecatedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is another deprecated test rule.")
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
#[violation]
pub struct RemovedTestRule;

impl Violation for RemovedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a removed test rule.")
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
#[violation]
pub struct AnotherRemovedTestRule;

impl Violation for AnotherRemovedTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a another removed test rule.")
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
#[violation]
pub struct RedirectedFromTestRule;

impl Violation for RedirectedFromTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a test rule that was redirected to another.")
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
#[violation]
pub struct RedirectedToTestRule;

impl Violation for RedirectedToTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a test rule that was redirected from another.")
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
#[violation]
pub struct RedirectedFromPrefixTestRule;

impl Violation for RedirectedFromPrefixTestRule {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Hey this is a test rule that was redirected to another by prefix.")
    }
}

impl TestRule for RedirectedFromPrefixTestRule {
    fn check() -> Option<Diagnostic> {
        Some(Diagnostic::new(Self {}, TextRange::default()))
    }
}
