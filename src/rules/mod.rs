/// A collection of all rules, and utilities to select a subset at runtime.
mod error;
mod filesystem;
#[macro_use]
mod macros;
mod modules;
mod precision;
mod style;
mod typing;
use crate::register_rules;
use crate::Rule;
use std::collections::{BTreeMap, BTreeSet};

register_rules! {
    (Category::Error, "E001", AST, error::syntax_error::SyntaxError, SyntaxError),
    (Category::Filesystem, "F001", PATH, filesystem::extensions::NonStandardFileExtension, NonStandardFileExtension),
    (Category::Style, "S001", TEXT, style::line_length::LineTooLong, LineTooLong),
    (Category::Style, "S101", TEXT, style::whitespace::TrailingWhitespace, TrailingWhitespace),
    (Category::Typing, "T001", AST, typing::implicit_typing::ImplicitTyping, ImplicitTyping),
    (Category::Typing, "T002", AST, typing::implicit_typing::InterfaceImplicitTyping, InterfaceImplicitTyping),
    (Category::Typing, "T003", AST, typing::implicit_typing::SuperfluousImplicitNone, SuperfluousImplicitNone),
    (Category::Typing, "T011", AST, typing::literal_kinds::LiteralKind, LiteralKind),
    (Category::Typing, "T012", AST, typing::literal_kinds::LiteralKindSuffix, LiteralKindSuffix),
    (Category::Typing, "T021", AST, typing::star_kinds::StarKind, StarKind),
    (Category::Precision, "P001", AST, precision::kind_suffixes::NoRealSuffix, NoRealSuffix),
    (Category::Precision, "P011", AST, precision::double_precision::DoublePrecision, DoublePrecision),
    (Category::Modules, "M001", AST, modules::external_functions::ExternalFunction, ExternalFunction),
    (Category::Modules, "M011", AST, modules::use_statements::UseAll, UseAll)
}

pub type RuleBox = Box<dyn Rule>;
pub type RuleSet = BTreeSet<String>;
pub type RuleMap = BTreeMap<String, RuleBox>;
pub type EntryPointMap = BTreeMap<String, Vec<(String, RuleBox)>>;

// Returns the full set of all rules.
pub fn full_ruleset() -> RuleSet {
    RuleSet::from_iter(CODES.iter().map(|x| x.to_string()))
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
