/// Contains utilities for categorising and defining rules.
// TODO Rules should be const creatable. Figure out str lifetimes.
// TODO Add RuleStatus, tagging rules as default or optional.
// TODO Add RuleRegistry, collecting all rules at compile time. A HashSet of active rules should be
//      determined at runtime depending on default/optional status and user choices.
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// The category of each rule defines the sort of problem it intends to solve.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// Rules that check for basic syntax errors -- the things that compilers should be
    /// able to tell you.
    SyntaxError,
    /// Rules for ensuring code is written in a way that minimises bugs and promotes
    /// maintainability.
    BestPractices,
    /// Rules for ensuring code follows certain style conventions. May be opinionated.
    CodeStyle,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SyntaxError => write!(f, "E"),
            Self::BestPractices => write!(f, "B"),
            Self::CodeStyle => write!(f, "S"),
        }
    }
}

/// The combination of a rule category and a unique identifying number.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Code {
    category: Category,
    number: u8,
}

impl Code {
    pub const fn new(category: Category, number: u8) -> Code {
        Code { category, number }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:03}", self.category, self.number)
    }
}

/// The type returned when a rule is violated.
#[derive(Eq)]
pub struct Violation {
    /// The line on which an error occurred.
    line: usize,
    /// The identity of the broken rule.
    code: Code,
    /// Description of the error.
    message: String,
}

impl Violation {
    pub fn new(line: usize, code: Code, message: &str) -> Violation {
        Violation {
            line,
            code,
            message: String::from(message),
        }
    }

    pub fn from_node(node: &tree_sitter::Node, code: Code, message: &str) -> Violation {
        Violation::new(node.start_position().row + 1, code, message)
    }
}

impl Ord for Violation {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.line, self.code).cmp(&(other.line, other.code))
    }
}

impl PartialOrd for Violation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Violation {
    fn eq(&self, other: &Self) -> bool {
        (self.line, self.code) == (other.line, other.code)
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {} {}", self.line, self.code, self.message)
    }
}

type TreeMethod = fn(&tree_sitter::Node) -> Vec<Violation>;
type StrMethod = fn(&str) -> Vec<Violation>;

/// The methods by which rules are enforced. Some rules act on individual lines of code,
/// some by reading a full file, and others by analysing the concrete syntax tree.
#[derive(PartialEq, Eq)]
pub enum Method {
    /// Methods that analyse the concrete syntax tree.
    Tree(TreeMethod),
    /// Methods that analyse individual lines of code, using regex or otherwise.
    Line(StrMethod),
    /// Methods that analyse multiple lines of code.
    File(StrMethod),
}

/// The definition of each rule.
#[derive(Eq)]
pub struct Rule {
    /// The unique identifier for this rule.
    code: Code,
    /// The method by which rules are enforced.
    method: Method,
    /// A description of what the rule does.
    description: String,
}

impl Rule {
    pub fn new(code: Code, method: Method, description: &str) -> Rule {
        Rule {
            code,
            method,
            description: String::from(description),
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.description)
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
            Code::new(Category::BestPractices, 23),
            Method::Line(|x: &str| {
                vec![Violation::new(1, Code::new(Category::BestPractices, 23), x)]
            }),
            &msg,
        );
        assert_eq!(mod_check.to_string(), format!("B023: {}", msg));
    }
}
