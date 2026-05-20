// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

pub mod message;
pub mod violation;

use std::{
    cmp::Ordering,
    ops::{Add, AddAssign},
};

use ruff_source_file::{SourceFile, SourceLocation};
use rustc_hash::FxHashMap;

use anyhow::Result;
use log::debug;
use ruff_text_size::{Ranged, TextRange};

use crate::{fix::FixTable, rules::Rule};

pub use violation::{AlwaysFixableViolation, FixAvailability, Violation, ViolationMetadata};

// Re-export some things from ruff
pub use ruff_diagnostics::{Applicability, Edit, Fix, IsolationLevel, SourceMap, SourceMarker};

#[derive(Debug, Default, PartialEq)]
pub struct Diagnostics {
    pub messages: Vec<Diagnostic>,
    pub fixed: FixMap,
}

impl Diagnostics {
    pub fn new(messages: Vec<Diagnostic>) -> Self {
        Self {
            messages,
            fixed: FixMap::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.fixed.is_empty()
    }
}

impl Add for Diagnostics {
    type Output = Diagnostics;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl AddAssign for Diagnostics {
    fn add_assign(&mut self, other: Self) {
        self.messages.extend(other.messages);
        self.fixed += other.fixed;
    }
}

/// A collection of fixes indexed by file path.
#[derive(Debug, Default, PartialEq)]
pub struct FixMap(FxHashMap<String, FixTable>);

impl FixMap {
    /// Returns `true` if there are no fixes in the map.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the fixes in the map, along with the file path.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FixTable)> {
        self.0.iter()
    }

    /// Returns an iterator over the fixes in the map.
    pub fn values(&self) -> impl Iterator<Item = &FixTable> {
        self.0.values()
    }
}

impl FromIterator<(String, FixTable)> for FixMap {
    fn from_iter<T: IntoIterator<Item = (String, FixTable)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .filter(|(_, fixes)| !fixes.is_empty())
                .collect(),
        )
    }
}

impl AddAssign for FixMap {
    fn add_assign(&mut self, rhs: Self) {
        for (filename, fixed) in rhs.0 {
            if fixed.is_empty() {
                continue;
            }
            let fixed_in_file = self.0.entry(filename).or_default();
            for (rule, count) in fixed {
                if count > 0 {
                    *fixed_in_file.entry(rule).or_default() += count;
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    /// The identifier of the diagnostic, used to align the diagnostic with a rule.
    name: &'static str,
    /// The message body to display to the user, to explain the diagnostic.
    body: String,
    /// The message to display to the user, to explain the suggested fix.
    suggestion: Option<String>,
    range: TextRange,
    /// The suggested fix for the violation.
    fix: Option<Fix>,
    rule: Rule,
    /// The rule code that was violated, expressed as a string.
    code: String,
    /// The file where an error was reported.
    ///
    /// Optional so we can delay setting it for now
    file: Option<SourceFile>,
}

impl Diagnostic {
    pub fn new<T: Violation>(kind: T, range: TextRange) -> Self {
        Self {
            name: T::rule().into(),
            body: Violation::message(&kind),
            suggestion: Violation::fix_title(&kind),
            range,
            fix: None,
            code: T::rule().noqa_code().to_string(),
            rule: T::rule(),
            file: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_file(mut self, file: SourceFile) -> Self {
        self.file = Some(file);
        self
    }

    /// Consumes `self` and returns a new `Diagnostic` with the given `fix`.
    #[inline]
    #[must_use]
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.set_fix(fix);
        self
    }

    /// Set the [`Fix`] used to fix the diagnostic.
    #[inline]
    pub fn set_fix(&mut self, fix: Fix) {
        self.fix = Some(fix);
    }

    /// Set the [`Fix`] used to fix the diagnostic, if the provided function returns `Ok`.
    /// Otherwise, log the error.
    #[inline]
    pub fn try_set_fix(&mut self, func: impl FnOnce() -> Result<Fix>) {
        match func() {
            Ok(fix) => self.fix = Some(fix),
            Err(err) => debug!("Failed to create fix for {}: {}", self.name, err),
        }
    }

    /// Set the [`Fix`] used to fix the diagnostic, if the provided function returns `Ok`.
    /// Otherwise, log the error.
    #[inline]
    pub fn try_set_optional_fix(&mut self, func: impl FnOnce() -> Result<Option<Fix>>) {
        match func() {
            Ok(None) => {}
            Ok(Some(fix)) => self.fix = Some(fix),
            Err(err) => debug!("Failed to create fix for {}: {}", self.name, err),
        }
    }

    /// Remove any previously set [`Fix`]
    #[inline]
    pub fn drop_fix(&mut self) {
        self.fix = None;
    }

    /// Consumes `self` and returns a new `Diagnostic` with the given `suggestion`.
    #[inline]
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: Option<String>) -> Self {
        self.suggestion = suggestion;
        self
    }

    /// Returns the name used to represent the diagnostic.
    pub fn name(&self) -> &str {
        self.rule.into()
    }

    /// Returns the message body to display to the user.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the rule code that was violated.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the fix suggestion for the violation.
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Returns the [`Fix`] for the message, if there is any.
    pub fn fix(&self) -> Option<&Fix> {
        self.fix.as_ref()
    }

    /// Returns `true` if the message contains a [`Fix`].
    pub fn fixable(&self) -> bool {
        self.fix().is_some()
    }

    /// Returns the [`Rule`] corresponding to the diagnostic message.
    pub fn rule(&self) -> Rule {
        self.rule
    }

    /// Returns the filename for the message.
    pub fn filename(&self) -> &str {
        self.source_file().name()
    }

    /// Computes the start source location for the message.
    pub fn compute_start_location(&self) -> SourceLocation {
        self.source_file()
            .to_source_code()
            .source_location(self.start())
    }

    /// Computes the end source location for the message.
    #[allow(dead_code)]
    pub fn compute_end_location(&self) -> SourceLocation {
        self.source_file()
            .to_source_code()
            .source_location(self.end())
    }

    /// Returns the [`SourceFile`] which the message belongs to.
    pub fn source_file(&self) -> &SourceFile {
        self.file.as_ref().expect("Must have file set")
    }

    /// Returns the URL for the rule documentation
    pub fn to_fortitude_url(&self) -> String {
        format!(
            "{}/en/stable/rules/{}",
            env!("CARGO_PKG_HOMEPAGE"),
            self.rule()
        )
    }
}

impl Ord for Diagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.file, self.range().start()).cmp(&(&other.file, other.range().start()))
    }
}

impl PartialOrd for Diagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ranged for Diagnostic {
    fn range(&self) -> TextRange {
        self.range
    }
}

#[cfg(test)]
pub fn test_diagnostic_builder(rule: Rule, body: &str, range: TextRange) -> Diagnostic {
    Diagnostic {
        name: rule.name(),
        body: body.to_string(),
        suggestion: None,
        range,
        fix: None,
        rule,
        code: rule.noqa_code().to_string(),
        file: None,
    }
}
