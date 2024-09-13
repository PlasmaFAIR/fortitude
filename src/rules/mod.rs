mod best_practices;
mod code_style;
mod error;
use crate::{Category, Code, Rule};
use std::collections::{HashMap, HashSet};
/// A collection of all rules, and utilities to select a subset at runtime.

pub type RuleBox = Box<dyn Rule>;
pub type RuleSet = HashSet<String>;
pub type RuleMap = HashMap<String, RuleBox>;

/// Create a new `Rule` given a rule code, expressed as a string.
pub fn build_rule(code_str: &str) -> anyhow::Result<RuleBox> {
    let code = Code::from(code_str)?;
    match code {
        Code {
            category: Category::SyntaxError,
            number: 1,
        } => Ok(Box::new(error::syntax_error::SyntaxError {})),
        Code {
            category: Category::BestPractices,
            number: 1,
        } => Ok(Box::new(
            best_practices::modules_and_programs::ExternalFunction {},
        )),
        Code {
            category: Category::BestPractices,
            number: 2,
        } => Ok(Box::new(best_practices::modules_and_programs::UseAll {})),
        Code {
            category: Category::BestPractices,
            number: 10,
        } => Ok(Box::new(best_practices::implicit_none::ImplicitTyping {})),
        Code {
            category: Category::BestPractices,
            number: 11,
        } => Ok(Box::new(
            best_practices::implicit_none::InterfaceImplicitTyping {},
        )),
        Code {
            category: Category::BestPractices,
            number: 12,
        } => Ok(Box::new(
            best_practices::implicit_none::SuperfluousImplicitNone {},
        )),
        Code {
            category: Category::BestPractices,
            number: 20,
        } => Ok(Box::new(best_practices::kinds::LiteralKind {})),
        Code {
            category: Category::BestPractices,
            number: 21,
        } => Ok(Box::new(best_practices::kinds::StarKind {})),
        Code {
            category: Category::BestPractices,
            number: 22,
        } => Ok(Box::new(best_practices::kinds::DoublePrecision {})),
        Code {
            category: Category::BestPractices,
            number: 23,
        } => Ok(Box::new(best_practices::kinds::NoRealSuffix {})),
        Code {
            category: Category::BestPractices,
            number: 24,
        } => Ok(Box::new(best_practices::kinds::LiteralKindSuffix {})),
        Code {
            category: Category::BestPractices,
            number: 60,
        } => Ok(Box::new(
            best_practices::filesystem::NonStandardFileExtension {},
        )),
        Code {
            category: Category::CodeStyle,
            number: 1,
        } => Ok(Box::new(code_style::whitespace::TrailingWhitespace {})),
        Code {
            category: Category::CodeStyle,
            number: 10,
        } => Ok(Box::new(code_style::line_length::LineTooLong {})),
        _ => {
            anyhow::bail!("Unknown rule code {}", code_str)
        }
    }
}

// Returns the full set of all rules.
pub fn full_ruleset() -> RuleSet {
    let all_rules = &[
        "E001", "B001", "B002", "B010", "B011", "B012", "B020", "B021", "B022", "B023", "B024",
        "B060", "S001", "S010",
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
