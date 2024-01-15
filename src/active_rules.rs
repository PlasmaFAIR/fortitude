use crate::best_practices::add_best_practices_rules;
use crate::rules::{Registry, Rule, Status};
use std::collections::HashMap;

pub fn get_all_rules() -> Registry {
    add_best_practices_rules(Registry::new())
}

type SubRegistry<'a> = HashMap<&'a str, &'a Rule>;

/// A subset of a `Registry` to be used in a given code run.
pub struct ActiveRules<'a> {
    registry: &'a Registry,
    active_rules: SubRegistry<'a>,
}

impl<'a> ActiveRules<'a> {
    pub fn new(registry: &'a Registry) -> ActiveRules<'a> {
        ActiveRules {
            registry,
            active_rules: SubRegistry::new(),
        }
    }

    /// Add a collection of rules to the active rule set.
    /// Unrecognised rules are ignored.
    pub fn add<T>(mut self, rules: T) -> Self
    where
        T: IntoIterator<Item = &'a String>,
    {
        for rule in rules {
            if let Some((k, v)) = self.registry.get_key_value(rule) {
                self.active_rules.insert(k.as_str(), v);
            }
        }
        self
    }

    /// Remove a collection of rules from the active rule set.
    /// Unrecognised rules are ignored.
    pub fn remove<T>(mut self, rules: T) -> Self
    where
        T: IntoIterator<Item = &'a String>,
    {
        for rule in rules {
            if let Some((k, _)) = self.registry.get_key_value(rule) {
                self.active_rules.remove(k.as_str());
            }
        }
        self
    }

    /// Add rules with a given status to the active rules set.
    pub fn with_status(mut self, status: Status) -> Self {
        for (k, v) in self.registry {
            if v.status == status {
                self.active_rules.insert(k.as_str(), v);
            }
        }
        self
    }
}

impl<'a> IntoIterator for ActiveRules<'a> {
    type Item = <SubRegistry<'a> as IntoIterator>::Item;
    type IntoIter = <SubRegistry<'a> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.active_rules.into_iter()
    }
}

impl<'a> IntoIterator for &'a ActiveRules<'a> {
    type Item = <&'a SubRegistry<'a> as IntoIterator>::Item;
    type IntoIter = <&'a SubRegistry<'a> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.active_rules.iter()
    }
}
