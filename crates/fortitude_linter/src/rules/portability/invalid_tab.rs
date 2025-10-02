use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// ## What it does
/// Checks for the use of tab characters as whitespace
///
/// ## Why is this bad?
/// Tabs are not part of the Fortran standard, and compilers may
/// reject the source if using a strict conformance mode (for example,
/// `gfortran -std=f2023 -Werror`).
#[derive(ViolationMetadata)]
pub(crate) struct InvalidTab;

impl Violation for InvalidTab {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;
    #[derive_message_formats]
    fn message(&self) -> String {
        "Invalid tab character".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Replace with spaces".to_string())
    }
}

pub fn check_invalid_tab(root: &Node, src: &SourceFile) -> Vec<Diagnostic> {
    src.source_text()
        .char_indices()
        .filter(|(_, c)| *c == '\t')
        .filter(|(index, _)| {
            if let Some(node) = root.named_descendant_for_byte_range(*index, *index) {
                !matches!(node.kind(), "comment" | "string_literal")
            } else {
                false
            }
        })
        .map(|(index, _)| {
            // TODO(peter): This doesn't render particularly well,
            // might be an issue with annotate-snippets?
            let start = TextSize::try_from(index).unwrap();
            let end = start + TextSize::new(1);
            let edit = Edit::replacement("    ".to_string(), start, start + TextSize::new(1));
            Diagnostic::new(InvalidTab, TextRange::new(start, end)).with_fix(Fix::unsafe_edit(edit))
        })
        .collect_vec()
}
