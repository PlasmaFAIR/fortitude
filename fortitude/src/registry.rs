use fortitude_macros::RuleNamespace;

use crate::rules::Rule;

// Rule categories and identity codes
// ----------------------------------
// Helps to sort rules into logical categories, and acts as a unique tag with which
// users can switch rules on and off.

pub trait AsRule {
    fn rule(&self) -> Rule;
}

impl Rule {
    pub fn from_code(code: &str) -> Result<Self, FromCodeError> {
        // TODO(peter): second var and lhs should be `code`
        let (category, _) = Category::parse_code(code).ok_or(FromCodeError::Unknown)?;
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

pub enum RuleCheckKind {
    Text,
    Path,
    AST,
}

/// The category of each rule defines the sort of problem it intends to solve.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, RuleNamespace)]
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
