use anyhow::Context;
use colored::Colorize;
use regex::Regex;
use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};
/// Contains utilities for defining and categorising rules.

// Rule categories and identity codes
// ----------------------------------
// Helps to sort rules into logical categories, and acts as a unique tag with which
// users can switch rules on and off.

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

impl Category {
    fn from(s: &str) -> anyhow::Result<Self> {
        match s {
            "B" => Ok(Self::BestPractices),
            "S" => Ok(Self::CodeStyle),
            "E" => Ok(Self::Error),
            _ => {
                anyhow::bail!("{} is not a rule category.", s)
            }
        }
    }
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
    pub category: Category,
    pub number: usize,
}

impl Code {
    pub const fn new(category: Category, number: usize) -> Self {
        Self { category, number }
    }

    pub fn from(code_str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(r"^([A-Z]+)(\d{3})$")?;
        let captures = re
            .captures(code_str)
            .context(format!("{} is not a valid error code.", code_str))?;
        let category_str = captures.get(1).map_or("", |x| x.as_str());
        let number_str = captures.get(2).map_or("", |x| x.as_str());
        let category = Category::from(category_str)?;
        let number = number_str.parse::<usize>()?;
        Ok(Code::new(category, number))
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:03}", self.category, self.number)
    }
}

// Violation type
// --------------

/// The location within a file at which a violation was detected
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViolationPosition {
    None,
    Line(usize),
    LineCol((usize, usize)),
}

// This type is created when a rule is broken. As not all broken rules come with a
// line number or column, it is recommended to use the `violation!` macro to create
// these, or the `from_node()` function when creating them from `tree_sitter` queries.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Violation {
    /// Description of the error.
    message: String,
    /// The location at which the error was detected.
    position: ViolationPosition,
}

impl Violation {
    pub fn new<T: AsRef<str>>(message: T, position: ViolationPosition) -> Self {
        Self {
            message: String::from(message.as_ref()),
            position,
        }
    }

    pub fn from_node<T: AsRef<str>>(message: T, node: &tree_sitter::Node) -> Self {
        let position = node.start_position();
        Violation::new(
            message,
            ViolationPosition::LineCol((position.row + 1, position.column + 1)),
        )
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    pub fn position(&self) -> ViolationPosition {
        self.position
    }
}

#[macro_export]
macro_rules! violation {
    ($msg:expr) => {
        $crate::core::Violation::new($msg, $crate::core::ViolationPosition::None)
    };
    ($msg:expr, $line:expr) => {
        $crate::core::Violation::new($msg, $crate::core::ViolationPosition::Line($line))
    };
    ($msg:expr, $line:expr, $col:expr) => {
        $crate::core::Violation::new(
            $msg,
            $crate::core::ViolationPosition::LineCol(($line, $col)),
        )
    };
}

// Rule methods
// ------------

/// The methods by which rules are enforced. Some rules act on individual lines of code,
/// some by reading a full file, and others by analysing the concrete syntax tree. All
/// rules must be associated with a `Method` via the `Rule` trait.
#[allow(dead_code, clippy::type_complexity)]
pub enum Method<'a> {
    /// Methods that work on just the path name of the file.
    Path(Box<dyn Fn(&Path) -> Option<Violation> + 'a>),
    /// Methods that analyse the syntax tree.
    Tree(Box<dyn Fn(&tree_sitter::Node, &str) -> Vec<Violation> + 'a>),
    /// Methods that analyse individual lines of code, using regex or otherwise.
    Line(Box<dyn Fn(usize, &str) -> Option<Violation> + 'a>),
    /// Methods that analyse multiple lines of code.
    MultiLine(Box<dyn Fn(&str) -> Vec<Violation> + 'a>),
}

// Rule trait
// ----------

/// Should be implemented for all rules.
pub trait Rule {
    /// Return a function pointer to the method associated with this rule.
    fn method(&self) -> Method;

    /// Return text explaining what the rule tests for, why this is important, and how
    /// the user might fix it.
    fn explain(&self) -> &str;
}

// Violation diagnostics
// ---------------------

/// Reports of each violation. They are pretty-printable and sortable.
#[derive(Eq)]
pub struct Diagnostic {
    /// The file where an error was reported.
    path: PathBuf,
    /// The rule code that was violated, expressed as a string.
    code: String,
    /// The specific violation detected
    violation: Violation,
}

impl Diagnostic {
    pub fn new<P, S>(path: P, code: S, violation: &Violation) -> Self
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            code: code.as_ref().to_string(),
            violation: violation.clone(),
        }
    }

    fn orderable(&self) -> (&Path, usize, usize, &str) {
        match self.violation.position() {
            ViolationPosition::None => (&self.path.as_path(), 0, 0, &self.code.as_str()),
            ViolationPosition::Line(line) => (&self.path.as_path(), line, 0, &self.code.as_str()),
            ViolationPosition::LineCol((line, col)) => {
                (&self.path.as_path(), line, col, &self.code.as_str())
            }
        }
    }
}

impl Ord for Diagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.orderable().cmp(&other.orderable())
    }
}

impl PartialOrd for Diagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.orderable() == other.orderable()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let path = self.path.to_string_lossy().bold();
        let code = self.code.bold().bright_red();
        let message = self.violation.message();
        match self.violation.position() {
            ViolationPosition::None => {
                write!(f, "{}: {} {}", path, code, message)
            }
            ViolationPosition::Line(line) => {
                write!(f, "{}:{}: {} {}", path, line, code, message)
            }
            ViolationPosition::LineCol((line, col)) => {
                write!(f, "{}:{}:{}: {} {}", path, line, col, code, message)
            }
        }
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

    // TODO Test diagnostics
}
