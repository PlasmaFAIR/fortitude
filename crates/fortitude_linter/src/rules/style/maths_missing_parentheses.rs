use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks if mathematical expressions are missing parentheses when operators have
/// different precedences.
///
/// ## Why is this bad?
/// Long or complex expressions can be difficult or confusing to read, especially when
/// mixing operators with different precedences. Adding parentheses can clarify the code
/// and make the author's intent clearer, reducing the likelihood of misunderstandings
/// or bugs.
///
/// ## Example
/// ```f90
/// x = 1. + 2. * 3. - 4. / 5.
/// ```
///
/// Use instead:
/// ```f90
/// x = 1. + (2. * 3.) - (4. / 5.)
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct MathsMissingParentheses {
    op1: String,
    op2: String,
}

impl AlwaysFixableViolation for MathsMissingParentheses {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { op1, op2 } = &self;
        format!("'{op1}' has higher precedence than '{op2}'; add parentheses for clarity")
    }

    fn fix_title(&self) -> String {
        "Add parentheses".to_string()
    }
}

#[derive(PartialEq)]
enum Operator {
    Exponential,
    Multiplication,
    Division,
    Addition,
    Subtraction,
}

impl Operator {
    fn precedence(&self) -> u8 {
        use Operator::*;
        match self {
            Exponential => 4,
            Multiplication | Division => 3,
            Addition | Subtraction => 2,
        }
    }
}

impl TryFrom<&str> for Operator {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Operator::*;
        match value {
            "**" => Ok(Exponential),
            "*" => Ok(Multiplication),
            "/" => Ok(Division),
            "-" => Ok(Addition),
            "+" => Ok(Subtraction),
            _ => Err("not a maths operator"),
        }
    }
}

impl From<Operator> for String {
    fn from(value: Operator) -> String {
        use Operator::*;
        match value {
            Exponential => "**",
            Multiplication => "*",
            Division => "/",
            Addition => "-",
            Subtraction => "+",
        }
        .to_string()
    }
}

impl AstRule for MathsMissingParentheses {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let text = src.source_text();
        let op: Operator = node.child(1)?.to_text(text)?.try_into().ok()?;

        let parent = node.parent()?;
        if !matches!(parent.kind(), "math_expression" | "unary_expression") {
            return None;
        }

        let edit = node.edit_replacement(src, format!("({})", node.to_text(text)?));
        let fix = Fix::safe_edit(edit);
        if op == Operator::Exponential && parent.kind() == "unary_expression" {
            let op2 = parent.child(0)?.to_text(text)?.to_string();
            let op1: String = op.into();
            return some_vec!(Diagnostic::from_node(Self { op1, op2 }, node).with_fix(fix));
        }

        let parent_op: Operator = parent.child(1)?.to_text(text)?.try_into().ok()?;

        if op.precedence() == parent_op.precedence() {
            return None;
        }

        let (op1, op2): (String, String) = if op.precedence() > parent_op.precedence() {
            (op.into(), parent_op.into())
        } else {
            (parent_op.into(), op.into())
        };

        some_vec!(Diagnostic::from_node(Self { op1, op2 }, node).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["math_expression"]
    }
}
