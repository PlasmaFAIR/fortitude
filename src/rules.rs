use crate::best_practices;
use crate::code_errors;
use crate::code_style;
use crate::core::{Category, Code, Rule};
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
            category: Category::Error,
            number: 1,
        } => Ok(Box::new(code_errors::SyntaxErrors {})),
        Code {
            category: Category::BestPractices,
            number: 1,
        } => Ok(Box::new(best_practices::UseModulesAndPrograms {})),
        Code {
            category: Category::BestPractices,
            number: 2,
        } => Ok(Box::new(best_practices::UseOnlyClause {})),
        Code {
            category: Category::BestPractices,
            number: 10,
        } => Ok(Box::new(
            best_practices::UseImplicitNoneModulesAndPrograms {},
        )),
        Code {
            category: Category::BestPractices,
            number: 11,
        } => Ok(Box::new(best_practices::UseImplicitNoneInterfaces {})),
        Code {
            category: Category::BestPractices,
            number: 12,
        } => Ok(Box::new(best_practices::AvoidSuperfluousImplicitNone {})),
        Code {
            category: Category::BestPractices,
            number: 20,
        } => Ok(Box::new(best_practices::AvoidNumberLiteralKinds {})),
        Code {
            category: Category::BestPractices,
            number: 21,
        } => Ok(Box::new(best_practices::AvoidNonStandardByteSpecifier {})),
        Code {
            category: Category::BestPractices,
            number: 22,
        } => Ok(Box::new(best_practices::AvoidDoublePrecision {})),
        Code {
            category: Category::BestPractices,
            number: 23,
        } => Ok(Box::new(best_practices::UseFloatingPointSuffixes {})),
        Code {
            category: Category::BestPractices,
            number: 24,
        } => Ok(Box::new(best_practices::AvoidNumberedKindSuffixes {})),
        Code {
            category: Category::BestPractices,
            number: 60,
        } => Ok(Box::new(best_practices::UseStandardFileExtensions {})),
        Code {
            category: Category::CodeStyle,
            number: 1,
        } => Ok(Box::new(code_style::AvoidTrailingWhitespace {})),
        Code {
            category: Category::CodeStyle,
            number: 10,
        } => Ok(Box::new(code_style::EnforceMaxLineLength {})),
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
