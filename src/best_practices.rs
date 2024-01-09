use crate::rules::{register_rule, Rule};
use std::collections::HashMap;

pub mod use_implicit_none;
pub mod use_modules;

pub fn add_best_practices_rules(registry: &mut HashMap<String, Rule>) {
    register_rule(registry, use_modules::use_modules());
    register_rule(registry, use_implicit_none::use_implicit_none());
    register_rule(registry, use_implicit_none::use_interface_implicit_none());
}
