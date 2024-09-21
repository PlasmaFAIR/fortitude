mod error;
mod filesystem;
mod modules;
mod precision;
mod style;
mod typing;
use crate::Rule;
use std::collections::{BTreeMap, BTreeSet};
/// A collection of all rules, and utilities to select a subset at runtime.

pub type RuleBox = Box<dyn Rule>;
pub type RuleSet = BTreeSet<String>;
pub type RuleMap = BTreeMap<String, RuleBox>;
pub type EntryPointMap = BTreeMap<String, Vec<(String, RuleBox)>>;

/// Create a new `Rule` given a rule code, expressed as a string.
pub fn build_rule(code: &str) -> anyhow::Result<RuleBox> {
    match code {
        "E001" => Ok(Box::new(error::syntax_error::SyntaxError {})),
        "F001" => Ok(Box::new(
            filesystem::extensions::NonStandardFileExtension {},
        )),
        "S001" => Ok(Box::new(style::line_length::LineTooLong {})),
        "S101" => Ok(Box::new(style::whitespace::TrailingWhitespace {})),
        "T001" => Ok(Box::new(typing::implicit_typing::ImplicitTyping {})),
        "T002" => Ok(Box::new(
            typing::implicit_typing::InterfaceImplicitTyping {},
        )),
        "T003" => Ok(Box::new(
            typing::implicit_typing::SuperfluousImplicitNone {},
        )),
        "T011" => Ok(Box::new(typing::literal_kinds::LiteralKind {})),
        "T012" => Ok(Box::new(typing::literal_kinds::LiteralKindSuffix {})),
        "T021" => Ok(Box::new(typing::star_kinds::StarKind {})),
        "P001" => Ok(Box::new(precision::kind_suffixes::NoRealSuffix {})),
        "P011" => Ok(Box::new(precision::double_precision::DoublePrecision {})),
        "M001" => Ok(Box::new(modules::external_functions::ExternalFunction {})),
        "M011" => Ok(Box::new(modules::use_statements::UseAll {})),
        _ => {
            anyhow::bail!("Unknown rule code {}", code)
        }
    }
}

// Returns the full set of all rules.
pub fn full_ruleset() -> RuleSet {
    let all_rules = &[
        "E001", "F001", "S001", "S101", "T001", "T002", "T003", "T011", "T012", "T021", "P001",
        "P011", "M001", "M011",
    ];
    RuleSet::from_iter(all_rules.iter().map(|x| x.to_string()))
}

/// Returns the set of rules that are activated by default, expressed as strings.
pub fn default_ruleset() -> RuleSet {
    // Currently all rules are activated by default.
    // Community feedback will be needed to determine a sensible set.
    full_ruleset()
}

pub fn rulemap(set: &RuleSet) -> anyhow::Result<RuleMap> {
    let mut rules = RuleMap::new();
    for code in set {
        let rule = build_rule(code)?;
        rules.insert(code.to_string(), rule);
    }
    Ok(rules)
}

/// Map tree-sitter node types to the rules that operate over them
pub fn entrypoint_map(set: &RuleSet) -> anyhow::Result<EntryPointMap> {
    let mut map = EntryPointMap::new();
    for code in set {
        let rule = build_rule(code)?;
        let entrypoints = rule.entrypoints();
        for entrypoint in entrypoints {
            match map.get_mut(entrypoint) {
                Some(rule_vec) => {
                    rule_vec.push((code.to_string(), build_rule(code)?));
                }
                None => {
                    map.insert(
                        entrypoint.to_string(),
                        vec![(code.to_string(), build_rule(code)?)],
                    );
                }
            }
        }
    }
    Ok(map)
}
