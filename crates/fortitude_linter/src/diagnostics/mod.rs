// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

pub mod diagnostic_message;
pub mod message;
pub mod violation;

use std::ops::{Add, AddAssign};

use rustc_hash::FxHashMap;

use anyhow::Result;
use log::debug;
use ruff_text_size::{Ranged, TextRange};
use tree_sitter::Node;

use crate::{fix::FixTable, rules::Rule, settings::CheckSettings, traits::TextRanged};

pub use diagnostic_message::DiagnosticMessage;
pub use violation::{AlwaysFixableViolation, FixAvailability, Violation, ViolationMetadata};

// Re-export some things from ruff
pub use ruff_diagnostics::{Applicability, Edit, Fix, IsolationLevel, SourceMap, SourceMarker};

#[derive(Debug, Default, PartialEq)]
pub struct Diagnostics {
    pub messages: Vec<DiagnosticMessage>,
    pub fixed: FixMap,
}

impl Diagnostics {
    pub fn new(messages: Vec<DiagnosticMessage>) -> Self {
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
    pub name: &'static str,
    /// The message body to display to the user, to explain the diagnostic.
    pub body: String,
    /// The message to display to the user, to explain the suggested fix.
    pub suggestion: Option<String>,
    pub range: TextRange,
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn new<T: Violation>(kind: T, range: TextRange) -> Self {
        Self {
            name: T::rule().into(),
            body: Violation::message(&kind),
            suggestion: Violation::fix_title(&kind),
            range,
            fix: None,
        }
    }

    pub fn from_node<T: Violation>(violation: T, node: &Node) -> Self {
        Self::new(violation, node.textrange())
    }

    pub fn from_node_if_rule_enabled<T: Violation>(
        settings: &CheckSettings,
        rule: Rule,
        violation: T,
        node: &Node,
    ) -> Option<Self> {
        if settings.rules.enabled(rule) {
            Some(Diagnostic::from_node(violation, node))
        } else {
            None
        }
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
}

impl Ranged for Diagnostic {
    fn range(&self) -> TextRange {
        self.range
    }
}
