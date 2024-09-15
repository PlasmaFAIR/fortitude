mod error;
mod filesystem;
mod modules;
mod style;
mod typing;
use crate::{Category, Code, Rule};
use std::collections::{BTreeMap, BTreeSet};
/// A collection of all rules, and utilities to select a subset at runtime.

pub type RuleBox = Box<dyn Rule>;
pub type RuleSet = BTreeSet<String>;
pub type RuleMap = BTreeMap<String, RuleBox>;

/// Create a new `Rule` given a rule code, expressed as a string.
pub fn build_rule(code_str: &str) -> anyhow::Result<RuleBox> {
    let code = Code::from(code_str)?;
    match code {
        Code {
            category: Category::Error,
            number: 1,
        } => Ok(Box::new(error::syntax_error::SyntaxError {})),
        Code {
            category: Category::FileSystem,
            number: 1,
        } => Ok(Box::new(
            filesystem::extensions::NonStandardFileExtension {},
        )),
        Code {
            category: Category::Style,
            number: 1,
        } => Ok(Box::new(style::line_length::LineTooLong {})),
        Code {
            category: Category::Style,
            number: 101,
        } => Ok(Box::new(style::whitespace::TrailingWhitespace {})),
        Code {
            category: Category::Typing,
            number: 1,
        } => Ok(Box::new(typing::literal_kinds::LiteralKind {})),
        Code {
            category: Category::Typing,
            number: 2,
        } => Ok(Box::new(typing::literal_kinds::LiteralKindSuffix {})),
        Code {
            category: Category::Typing,
            number: 11,
        } => Ok(Box::new(typing::star_kinds::StarKind {})),
        Code {
            category: Category::Typing,
            number: 21,
        } => Ok(Box::new(typing::real_precision::DoublePrecision {})),
        Code {
            category: Category::Typing,
            number: 22,
        } => Ok(Box::new(typing::real_precision::NoRealSuffix {})),
        Code {
            category: Category::Typing,
            number: 31,
        } => Ok(Box::new(typing::implicit_typing::ImplicitTyping {})),
        Code {
            category: Category::Typing,
            number: 32,
        } => Ok(Box::new(
            typing::implicit_typing::InterfaceImplicitTyping {},
        )),
        Code {
            category: Category::Typing,
            number: 33,
        } => Ok(Box::new(
            typing::implicit_typing::SuperfluousImplicitNone {},
        )),
        Code {
            category: Category::Modules,
            number: 1,
        } => Ok(Box::new(modules::external_functions::ExternalFunction {})),
        Code {
            category: Category::Modules,
            number: 11,
        } => Ok(Box::new(modules::use_statements::UseAll {})),
        _ => {
            anyhow::bail!("Unknown rule code {}", code_str)
        }
    }
}

// Returns the full set of all rules.
pub fn full_ruleset() -> RuleSet {
    let all_rules = &[
        "E001", "F001", "S001", "S101", "T001", "T002", "T011", "T021", "T022", "T031", "T032",
        "T033", "M001", "M011",
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
