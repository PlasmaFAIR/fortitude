use itertools::Itertools;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

use crate::settings::CheckSettings;

/// ## What it does
/// Checks for the use of tab characters as whitespace
///
/// ## Why is this bad?
/// Tabs are not part of the Fortran standard, and compilers may
/// reject the source if using a strict conformance mode (for example,
/// `gfortran -std=f2023 -Werror`).
///
/// ## Options
/// - `check.invalid-tab.indent-width`
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

pub fn check_invalid_tab(
    root: &Node,
    src: &SourceFile,
    settings: &CheckSettings,
) -> Vec<Diagnostic> {
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
            let start = TextSize::try_from(index).unwrap();
            let range = TextRange::new(start, start + TextSize::new(1));
            let width = settings.invalid_tab.indent_width.as_usize();
            let indent = format!("{:width$}", " ");
            let edit = Edit::range_replacement(indent, range);
            Diagnostic::new(InvalidTab, range).with_fix(Fix::unsafe_edit(edit))
        })
        .collect_vec()
}

pub mod settings {
    use crate::{display_settings, line_width::IndentWidth};
    use ruff_macros::CacheKey;
    use std::fmt::Display;

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub indent_width: IndentWidth,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.invalid_tab",
                fields = [self.indent_width]
            }
            Ok(())
        }
    }
}
