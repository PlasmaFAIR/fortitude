use colored::Colorize;
use std::collections::HashMap;
/// Contains utilities for categorising and defining rules.
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// The category of each rule defines the sort of problem it intends to solve.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Category {
    /// Rules for ensuring code is written in a way that minimises bugs and promotes
    /// maintainability.
    BestPractices,
    /// Rules for ensuring code follows certain style conventions. May be opinionated.
    CodeStyle,
    /// Used to indicate a failure to process or parse a file.
    Error,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::BestPractices => "B",
            Self::CodeStyle => "S",
            Self::Error => "E",
        };
        write!(f, "{}", s)
    }
}

/// The combination of a rule category and a unique identifying number.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Violation {
    /// The file in which an issue was found.
    path: PathBuf,
    /// The line on which an issue was found.
    line: usize,
    /// The rule code that triggered the violation.
    code: Code,
    /// Description of the error.
    message: String,
}

impl Violation {
    pub fn new(path: &Path, line: usize, code: Code, message: &str) -> Violation {
        Violation {
            path: PathBuf::from(path),
            line,
            code,
            message: String::from(message),
        }
    }

    pub fn from_node(
        path: &Path,
        node: &tree_sitter::Node,
        code: Code,
        message: &str,
    ) -> Violation {
        Violation::new(path, node.start_position().row + 1, code, message)
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} {}",
            self.path.to_string_lossy().bold(),
            self.line,
            self.code.to_string().bold().bright_red(),
            self.message
        )
    }
}

type PathMethod = fn(Code, &Path) -> Vec<Violation>;
type TreeMethod = fn(Code, &Path, &tree_sitter::Node, &str) -> Vec<Violation>;
type StrMethod = fn(Code, &Path, &str) -> Vec<Violation>;

/// The methods by which rules are enforced. Some rules act on individual lines of code,
/// some by reading a full file, and others by analysing the concrete syntax tree.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Method {
    /// Methods that work on just the path name of the file.
    Path(PathMethod),
    /// Methods that analyse the concrete syntax tree.
    Tree(TreeMethod),
    /// Methods that analyse individual lines of code, using regex or otherwise.
    Line(StrMethod),
    /// Methods that analyse multiple lines of code.
    File(StrMethod),
}

/// A way to tag rules as being on or off by default.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Rules that are 'on' by default.
    Standard,
    /// Rules that are 'off' by default.
    Optional,
}

/// The definition of each rule.
#[derive(Eq, Clone)]
pub struct Rule {
    /// The unique identifier for this rule.
    code: Code,
    // The method used to enforce the rule.
    method: Method,
    /// A description of what the rule does.
    description: String,
    /// Whether the rule should be switched on by default.
    status: Status,
}

impl Rule {
    pub fn new(code: Code, method: Method, description: &str, status: Status) -> Rule {
        Rule {
            code,
            method,
            description: String::from(description),
            status,
        }
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn status(&self) -> Status {
        self.status
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

pub type Registry = HashMap<String, Rule>;

/// Add a rule to a registry, using the string representation of its code as the key.
pub fn register_rule(registry: &mut Registry, rule: Rule) {
    registry.insert(rule.code.to_string(), rule);
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
        let rule = Rule::new(
            Code::new(Category::BestPractices, 23),
            Method::Line(|code: Code, path: &Path, x: &str| vec![Violation::new(path, 1, code, x)]),
            "hello world",
            Status::Standard,
        );
        assert_eq!(rule.to_string(), "B023: hello world");
    }

    #[test]
    fn test_registry() {
        let mut registry = Registry::new();
        let rule = Rule::new(
            Code::new(Category::BestPractices, 23),
            Method::Line(|code: Code, path: &Path, x: &str| vec![Violation::new(path, 1, code, x)]),
            "hello world",
            Status::Standard,
        );
        register_rule(&mut registry, rule);
        let rule = Rule::new(
            Code::new(Category::Error, 42),
            Method::Line(|code: Code, path: &Path, x: &str| vec![Violation::new(path, 1, code, x)]),
            "foo bar",
            Status::Optional,
        );
        register_rule(&mut registry, rule);
        assert_eq!(registry.contains_key("B023"), true);
        assert_eq!(registry.contains_key("E042"), true);
    }
}
