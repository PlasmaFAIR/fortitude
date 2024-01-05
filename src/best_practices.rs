use std::collections::HashSet;
use crate::rules::Rule;

pub mod use_modules;

pub fn add_best_practices_rules(set: &mut HashSet<Rule>) {
    set.insert(use_modules::use_modules());
}
