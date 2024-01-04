use std::fmt;
use std::path::PathBuf;

/// The category of each rule defines the sort of problem it intends to solve.
pub enum RuleCategory {
    /// Rules for ensuring code is written in a way that minimises bugs and promotes
    /// maintainability.
    BestPractices,
    /// Rules for ensuring code follows certain style conventions. May be opinionated.
    CodeStyle,
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BestPractices => write!(f, "B"),
            Self::CodeStyle => write!(f, "S"),
        }
    }
}

/// The combination of a rule category and a unique identifying number.
pub struct RuleCode {
    category: RuleCategory,
    number: u8,
}

impl RuleCode {
    fn new(category: RuleCategory, number: u8) -> RuleCode {
        RuleCode { category: category, number: number }
    }
}

impl fmt::Display for RuleCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:03}", self.category, self.number)
    }
}

/// The methods by which rules are enforced. Some rules act on individual lines of code,
/// some by reading a full file, and others by analysing the concrete syntax tree.
pub enum RuleMethod {
    /// Methods that analyse the concrete syntax tree.
    Tree,
    /// Methods that analyse individual lines of code, using regex or otherwise.
    Line,
}

/// The definition of each rule.
pub struct Rule {
    /// The unique identifier for this rule.
    code: RuleCode,
    /// The method by which rules are enforced.
    method: RuleMethod,
    /// A description of what the rule does.
    description: String,
}

impl Rule {
    fn new(category: RuleCategory, number: u8, method: RuleMethod, description: &str)
        -> Rule {
        Rule {
            code: RuleCode::new(category, number),
            method: method,
            description: String::from(description),
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.description)
    }
}

/// The type returned when a rule is violated.
pub struct RuleViolation {
    /// The file in which the error occurred.
    file: PathBuf,
    /// The line on which an error occurred.
    line: usize,
    /// The unique identifier for this rule.
    code: RuleCode,
    /// Description of the error.
    description: String,
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} line {}: {} {}", self.file.display(), self.line, self.code, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_code() {
        let b001 = RuleCode::new(RuleCategory::BestPractices, 1);
        assert_eq!(b001.to_string(), "B001");
        let c120 = RuleCode::new(RuleCategory::CodeStyle, 120);
        assert_eq!(c120.to_string(), "S120");
    }

    #[test]
    fn test_rule() {
        let msg = "Ensure functions and subroutines are contained within modules";
        let mod_check = Rule::new(
            RuleCategory::BestPractices,
            23, 
            RuleMethod::Tree,
            &msg,
        );
        assert_eq!(mod_check.to_string(), format!("B023: {}", msg));
    }
}
