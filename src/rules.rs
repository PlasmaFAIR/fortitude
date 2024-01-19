use crate::best_practices;
use crate::code_errors;
use crate::code_style;
use crate::core::{Category, Code, Rule};
use crate::settings::Settings;
use std::collections::{HashMap, HashSet};
/// A collection of all rules, and utilities to select a subset at runtime.

pub type RuleBox = Box<dyn Rule>;
pub type RuleSet = HashSet<String>;
pub type RuleMap = HashMap<String, RuleBox>;

/// Create a new `Rule` given a rule code, expressed as a string.
pub fn build_rule(code_str: &str, settings: &Settings) -> anyhow::Result<RuleBox> {
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
        } => Ok(Box::new(best_practices::UseFloatingPointSuffixes::new(
            settings,
        ))),
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
        } => Ok(Box::new(code_style::AvoidTrailingWhitespace::new()?)),
        Code {
            category: Category::CodeStyle,
            number: 10,
        } => Ok(Box::new(code_style::EnforceMaxLineLength::new(settings)?)),
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
    let defaults = &[
        "E001", "B001", "B010", "B011", "B020", "B021", "B022", "B023", "B024", "B060", "S001",
        "S010",
    ];
    RuleSet::from_iter(defaults.iter().map(|x| x.to_string()))
}

/// Returns the set of extra rules that are activated under `--strict` mode.
pub fn strict_ruleset() -> RuleSet {
    let stricts = &["B002", "B012"];
    RuleSet::from_iter(stricts.iter().map(|x| x.to_string()))
}

pub fn rulemap(set: &RuleSet, settings: &Settings) -> anyhow::Result<RuleMap> {
    let mut rules = RuleMap::new();
    for code in set {
        let rule = build_rule(code, settings)?;
        rules.insert(code.to_string(), rule);
    }
    Ok(rules)
}
