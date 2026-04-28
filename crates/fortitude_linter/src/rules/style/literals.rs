use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What it does
/// Checks for floating point literals that begin or end in a bare decimal point,
/// such as `.5` or `2.`.
///
/// ## Why is this bad?
/// Floating point literals that begin or end in a bare decimal point can be
/// missed and may lead to confusion. For example, `.5` could be
/// misread as `5`. It is generally recommended to include a leading zero
/// before the decimal point and a trailing zero after the decimal point for
/// clarity, such as `0.5` and `2.0`.
#[derive(ViolationMetadata)]
pub(crate) struct BareDecimal {
    literal: String,
    preferred: String,
    is_trailing: bool,
}

impl AlwaysFixableViolation for BareDecimal {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            is_trailing,
            literal,
            ..
        } = self;
        let trailing = if *is_trailing { "trailing" } else { "leading" };
        format!("{trailing} decimal point in `real` literal `{literal}`")
    }

    fn fix_title(&self) -> String {
        let Self { preferred, .. } = self;
        format!("Prefer `{preferred}`")
    }
}

impl AstRule for BareDecimal {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let txt = node.to_text(src.source_text())?;
        // Three cases to match:
        // 1. Leading decimal point, e.g. `.5`
        // 2. Trailing decimal point, e.g. `2.`
        // 3. Decimal point before exponent, e.g. `1.d2` or `1.e-3`
        let pos = txt.bytes().position(|b| b == b'.')?;
        let (preferred, is_trailing, edit) = if pos == 0 {
            // Case 1: Leading decimal point
            let preferred = format!("0{txt}");
            let is_trailing = false;
            let edit = Edit::insertion("0".to_string(), node.start_textsize());
            (preferred, is_trailing, edit)
        } else if pos == txt.len() - 1 {
            // Case 2: Trailing decimal point
            let preferred = format!("{txt}0");
            let is_trailing = true;
            let edit = Edit::insertion("0".to_string(), node.end_textsize());
            (preferred, is_trailing, edit)
        } else {
            // Case 3: Decimal point before exponent
            // Must additionally check that the character after the decimal
            // point is not a digit, to avoid matching things like `1.23`
            if txt.as_bytes().get(pos + 1)?.is_ascii_digit() {
                return None;
            }
            let preferred = format!("{}.0{}", &txt[..pos], &txt[pos + 1..]);
            let start = node.start_textsize();
            let insert_pos = start + TextSize::from((pos + 1) as u32);
            let is_trailing = true;
            let edit = Edit::insertion("0".to_string(), insert_pos);
            (preferred, is_trailing, edit)
        };
        some_vec![
            Diagnostic::from_node(
                BareDecimal {
                    literal: txt.to_string(),
                    preferred,
                    is_trailing
                },
                node
            )
            .with_fix(Fix::safe_edit(edit))
        ]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["number_literal"]
    }
}
