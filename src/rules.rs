/// Contains utilities for categorising and defining rules.
///
/// TODO: Add global HashMap rules registry that each rule will be added to.

use std::fmt;

/// The category of each rule defines the sort of problem it intends to solve.
pub enum Category {
    /// Rules for ensuring code is written in a way that minimises bugs and promotes
    /// maintainability.
    BestPractices,
    /// Rules for ensuring code follows certain style conventions. May be opinionated.
    CodeStyle,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BestPractices => write!(f, "B"),
            Self::CodeStyle => write!(f, "S"),
        }
    }
}

/// The combination of a rule category and a unique identifying number.
pub struct Code {
    category: Category,
    number: u8,
}

impl Code {
    pub fn new(category: Category, number: u8) -> Code {
        Code{category, number}
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:03}", self.category, self.number)
    }
}

/// The methods by which rules are enforced. Some rules act on individual lines of code,
/// some by reading a full file, and others by analysing the concrete syntax tree.
/// 
/// TODO: Method should use function types to specify the required signatures.
pub enum Method {
    /// Methods that analyse the concrete syntax tree.
    Tree,
    /// Methods that analyse individual lines of code, using regex or otherwise.
    Line,
}

/// The definition of each rule.
pub struct Rule {
    /// The unique identifier for this rule.
    code: Code,
    /// The method by which rules are enforced.
    method: Method,
    /// A description of what the rule does.
    description: String,
}

impl Rule {
    pub fn new(category: Category, number: u8, method: Method, description: &str) -> Rule {
        Rule {
            code: Code::new(category, number),
            method,
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
pub struct Violation {
    /// The line on which an error occurred.
    line: usize,
    /// Description of the error.
    message: String,
}

impl Violation {
    pub fn new(line: usize, message: &str) -> Violation {
        Violation{line, message: String::from(message)}
    }

    pub fn from_node(node: &tree_sitter::Node, message: &str) -> Violation {
        Violation::new(node.start_position().row + 1, message)
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.line, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_code() {
        let b001 = Code::new(Category::BestPractices, 1);
        assert_eq!(b001.to_string(), "B001");
        let c120 = Code::new(Category::CodeStyle, 120);
        assert_eq!(c120.to_string(), "S120");
    }

    #[test]
    fn test_rule() {
        let msg = "Ensure functions and subroutines are contained within modules";
        let mod_check = Rule::new(
            Category::BestPractices,
            23, 
            Method::Tree,
            &msg,
        );
        assert_eq!(mod_check.to_string(), format!("B023: {}", msg));
    }
}
