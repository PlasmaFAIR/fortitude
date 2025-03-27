/// Defines rules that govern the use of keywords.
use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

enum ElseClause {
    ElseIf,
    ElseWhere,
}

/// ## What it does
/// Checks for the use of `elseif` instead of `else if` and `elsewhere` instead
/// of `else where`.
///
/// ## Why is this bad?
/// TODO Needs a better explanation than "because I prefer it this way"
#[derive(ViolationMetadata)]
pub struct ElseClauseMissingSpace {
    entity: ElseClause,
}

impl AlwaysFixableViolation for ElseClauseMissingSpace {
    #[derive_message_formats]
    fn message(&self) -> String {
        match self.entity {
            ElseClause::ElseIf => "Prefer 'else if' over 'elseif'".to_string(),
            ElseClause::ElseWhere => "Prefer 'else where' over 'elsewhere'".to_string(),
        }
    }

    fn fix_title(&self) -> String {
        "Add missing space".to_string()
    }
}

impl AstRule for ElseClauseMissingSpace {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let first_child = node.child(0)?;
        let text = first_child.to_text(src.source_text())?;
        let entity = match text.to_lowercase().as_str() {
            "elseif" => ElseClause::ElseIf,
            "elsewhere" => ElseClause::ElseWhere,
            _ => return None,
        };
        let space_pos = TextSize::try_from(node.start_byte() + 4).unwrap();
        let fix = Fix::safe_edit(Edit::insertion(" ".to_string(), space_pos));
        some_vec!(Diagnostic::from_node(Self { entity }, &first_child).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["elseif_clause", "elsewhere_clause"]
    }
}
