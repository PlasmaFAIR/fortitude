use crate::best_practices::add_best_practices_rules;
use crate::rules::{Registry, Status};
/// Utility functions to manage the active rule set for a given run.

/// Collect all possible rules into a single registry.
pub fn get_all_rules() -> Registry {
    let mut registry = Registry::new();
    add_best_practices_rules(&mut registry);
    registry
}

/// Add a collection of rules to the active rule set.
/// Unrecognised rules are ignored.
pub fn add_rules(registry: &Registry, subregistry: &mut Registry, rule_codes: &Vec<String>) {
    for rule in rule_codes {
        if let Some(v) = registry.get(rule) {
            subregistry.insert(rule.clone(), v.clone());
        }
    }
}

/// Remove a collection of rules from the active rule set.
/// Unrecognised rules are ignored.
pub fn remove_rules(subregistry: &mut Registry, rule_codes: &Vec<String>) {
    for rule in rule_codes {
        subregistry.remove(rule);
    }
}

/// Add rules with a given status to the active rules set.
pub fn get_rules_with_status(status: Status, registry: &Registry) -> Vec<String> {
    let mut result = Vec::new();
    for (k, v) in registry {
        if v.status() == status {
            result.push(k.clone());
        }
    }
    result
}
