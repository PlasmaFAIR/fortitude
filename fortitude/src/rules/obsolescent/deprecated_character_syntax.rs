use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use lazy_regex::regex_captures;
use ruff_diagnostics::{Diagnostic, Fix, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What does it do?
/// Checks for outdated declarations of `character*N`
///
/// ## Why is this bad?
/// The syntax `character*N` has been replaced by `character(len=N)` in modern
/// Fortran. Prefer the second form.
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedCharacterSyntax {
    original: String,
    dtype: String,
    length: String,
}

impl Violation for DeprecatedCharacterSyntax {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { original, .. } = self;
        format!("'{original}' uses deprecated syntax")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { dtype, length, .. } = self;
        Some(format!("Replace with '{dtype}(len={length})'"))
    }
}

impl AstRule for DeprecatedCharacterSyntax {
    fn check(
        _settings: &Settings,
        node: &Node,
        source_file: &SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let src = source_file.source_text();

        // Rule only applies to `character`.
        // Expect child(0) to always be present.
        let dtype = node.child(0)?;
        if dtype.kind() != "character" {
            return None;
        }

        // If 'kind' field isn't present, exit early
        let kind = node.child_by_field_name("kind")?;
        let kind_text = kind.to_text(src)?;

        // If kind does not start with '*', exit early
        if !kind_text.starts_with('*') {
            return None;
        }

        // The '*' should be followed by:
        // - An integer literal
        // - '(*)'
        // - An integer expression within parentheses
        // To test for the latter, we need to consider two cases. If some
        // arithmetic takes place, we need to check for a math_expression node
        // within the kind node. If there isn't, and the kind is instead something
        // like '(3)', '(N)', or '((((4))))', we need to handle that with regex.
        let length = match kind
            .named_descendants()
            .find(|n| n.kind() == "math_expression")
        {
            Some(math_expression) => math_expression.to_text(src)?.to_string(),
            _ => {
                // If there is no math_expression node, this may be an integer
                // literal or '(*)', or it may also be something like '(N)', or
                // '((((4))))'.
                // An arbitrary amount of whitespace is permitted.
                // Strangely, plain '*N' is not allowed.
                let (_, length, star, expression) = regex_captures!(
                    r#"^\*(?:\s*(\d+)|\s*\(\s*(\*)\s*\)|\([\s\(]*([[:word:]]+)[\s\)]*\))$"#,
                    kind_text
                )?;
                // Only one of length, star, or expression should be present. The others will be empty.
                if !length.is_empty() {
                    length.to_string()
                } else if !star.is_empty() {
                    star.to_string()
                } else if !expression.is_empty() {
                    expression.to_string()
                } else {
                    panic!("This should not happen");
                }
            }
        };

        let original = node.to_text(src)?.to_string();
        let dtype = dtype.to_text(src)?.to_string();
        let replacement = format!("{}(len={})", dtype, length);
        let fix = Fix::safe_edit(node.edit_replacement(source_file, replacement));
        some_vec![Diagnostic::from_node(
            Self {
                original,
                dtype,
                length
            },
            node
        )
        .with_fix(fix)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["intrinsic_type"]
    }
}
