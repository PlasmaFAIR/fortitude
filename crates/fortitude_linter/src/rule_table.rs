// Taken from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter},
    sync::OnceLock,
};

use crate::{display_settings, rules::AstRuleEnum};
use ruff_macros::CacheKey;

use crate::registry::{Rule, RuleSet, RuleSetIterator};

/// A table to keep track of which rules are enabled and whether they should be fixed.
#[derive(Debug, Clone, CacheKey, Default)]
pub struct RuleTable {
    /// Maps rule codes to a boolean indicating if the rule should be fixed.
    enabled: RuleSet,
    should_fix: RuleSet,
    #[cache_key(ignore)]
    ast_entrypoints: OnceLock<BTreeMap<&'static str, Vec<AstRuleEnum>>>,
}

impl RuleTable {
    /// Creates a new empty rule table.
    pub const fn empty() -> Self {
        Self {
            enabled: RuleSet::empty(),
            should_fix: RuleSet::empty(),
            ast_entrypoints: OnceLock::new(),
        }
    }

    /// Returns whether the given rule should be checked.
    #[inline]
    pub const fn enabled(&self, rule: Rule) -> bool {
        self.enabled.contains(rule)
    }

    /// Returns whether any of the given rules should be checked.
    #[inline]
    pub const fn any_enabled(&self, rules: &[Rule]) -> bool {
        self.enabled.any(rules)
    }

    /// Returns whether violations of the given rule should be fixed.
    #[inline]
    pub const fn should_fix(&self, rule: Rule) -> bool {
        self.should_fix.contains(rule)
    }

    /// Returns an iterator over all enabled rules.
    pub fn iter_enabled(&self) -> RuleSetIterator {
        self.enabled.iter()
    }

    /// Enables the given rule.
    #[inline]
    pub fn enable(&mut self, rule: Rule, should_fix: bool) {
        self.enabled.insert(rule);

        if should_fix {
            self.should_fix.insert(rule);
        }
        // Invalidate the AST entrypoints
        self.ast_entrypoints.take();
    }

    /// Disables the given rule.
    #[inline]
    pub fn disable(&mut self, rule: Rule) {
        self.enabled.remove(rule);
        self.should_fix.remove(rule);
        // Invalidate the AST entrypoints
        self.ast_entrypoints.take();
    }

    /// Return a mapping of AST entrypoints to lists of the rules and codes that
    /// operate on them.
    pub fn ast_entrypoints(&self) -> &BTreeMap<&'static str, Vec<AstRuleEnum>> {
        self.ast_entrypoints.get_or_init(|| {
            let mut map: BTreeMap<&'static str, Vec<_>> = BTreeMap::new();

            self.iter_enabled()
                .filter_map(|rule| TryFrom::try_from(rule).ok())
                .for_each(|rule: AstRuleEnum| {
                    for entrypoint in rule.entrypoints() {
                        map.entry(entrypoint).or_default().push(rule);
                    }
                });

            map
        })
    }
}

impl Display for RuleTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        display_settings! {
            formatter = f,
            namespace = "linter.rules",
            fields = [
                self.enabled,
                self.should_fix
            ]
        }
        Ok(())
    }
}

impl FromIterator<Rule> for RuleTable {
    fn from_iter<T: IntoIterator<Item = Rule>>(iter: T) -> Self {
        let rules = RuleSet::from_iter(iter);
        Self {
            enabled: rules.clone(),
            should_fix: rules,
            ast_entrypoints: OnceLock::new(),
        }
    }
}
