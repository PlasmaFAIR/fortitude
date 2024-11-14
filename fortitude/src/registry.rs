use fortitude_macros::RuleNamespace;
use std::str::FromStr; // Needed by strum_macros

pub use crate::rules::Rule;

// Rule categories and identity codes
// ----------------------------------
// Helps to sort rules into logical categories, and acts as a unique tag with which
// users can switch rules on and off.

pub trait AsRule {
    fn rule(&self) -> Rule;
}

impl Rule {
    pub fn from_code(code: &str) -> Result<Self, FromCodeError> {
        let (category, code) = Category::parse_code(code).ok_or(FromCodeError::Unknown)?;
        category
            .all_rules()
            .find(|rule| rule.noqa_code().suffix() == code)
            .ok_or(FromCodeError::Unknown)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FromCodeError {
    #[error("unknown rule code")]
    Unknown,
}

/// The category of each rule defines the sort of problem it intends to solve.
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Hash,
    PartialOrd,
    Ord,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::IntoStaticStr,
    RuleNamespace,
)]
#[repr(u16)]
#[strum(serialize_all = "kebab-case")]
pub enum Category {
    /// Failure to parse a file.
    #[prefix = "E"]
    Error,
    /// Violation of style conventions.
    #[prefix = "S"]
    Style,
    /// Misuse of types and kinds.
    #[prefix = "T"]
    Typing,
    /// Failure to use modules or use them appropriately.
    #[prefix = "M"]
    Modules,
    /// Best practices for setting floating point precision.
    #[prefix = "P"]
    Precision,
    /// Check path names, directory structures, etc.
    #[prefix = "F"]
    Filesystem,
}

pub trait RuleNamespace: Sized {
    /// Returns the prefix that every single code that fortitude uses to identify
    /// rules from this category starts with.
    fn common_prefix(&self) -> &'static str;

    /// Attempts to parse the given rule code. If the prefix is recognized
    /// returns the respective variant along with the code with the common
    /// prefix stripped.
    fn parse_code(code: &str) -> Option<(Self, &str)>;

    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    #[allow(dead_code)]
    fn description(&self) -> &'static str;
}

pub mod clap_completion {
    use clap::builder::{PossibleValue, TypedValueParser, ValueParserFactory};
    use strum::IntoEnumIterator;

    use crate::registry::Rule;

    #[derive(Clone)]
    pub struct RuleParser;

    impl ValueParserFactory for Rule {
        type Parser = RuleParser;

        fn value_parser() -> Self::Parser {
            RuleParser
        }
    }

    impl TypedValueParser for RuleParser {
        type Value = Rule;

        fn parse_ref(
            &self,
            cmd: &clap::Command,
            arg: Option<&clap::Arg>,
            value: &std::ffi::OsStr,
        ) -> Result<Self::Value, clap::Error> {
            let value = value
                .to_str()
                .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;

            Rule::from_code(value).map_err(|_| {
                let mut error =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    error.insert(
                        clap::error::ContextKind::InvalidArg,
                        clap::error::ContextValue::String(arg.to_string()),
                    );
                }
                error.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(value.to_string()),
                );
                error
            })
        }

        fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
            Some(Box::new(Rule::iter().map(|rule| {
                let name = rule.noqa_code().to_string();
                let help = rule.as_ref().to_string();
                PossibleValue::new(name).help(help)
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use strum::IntoEnumIterator;

    use super::{Category, Rule, RuleNamespace};

    #[test]
    fn documentation() {
        for rule in Rule::iter() {
            assert!(
                rule.explanation().is_some(),
                "Rule {} is missing documentation",
                rule.as_ref()
            );
        }
    }

    #[test]
    fn check_code_serialization() {
        for rule in Rule::iter() {
            assert!(
                Rule::from_code(&format!("{}", rule.noqa_code())).is_ok(),
                "{rule:?} could not be round-trip serialized."
            );
        }
    }

    #[test]
    fn category_parse_code() {
        for rule in Rule::iter() {
            let code = format!("{}", rule.noqa_code());
            let (category, rest) =
                Category::parse_code(&code).unwrap_or_else(|| panic!("couldn't parse {code:?}"));
            assert_eq!(code, format!("{}{rest}", category.common_prefix()));
        }
    }

    #[test]
    fn rule_size() {
        assert_eq!(2, size_of::<Rule>());
    }
}
