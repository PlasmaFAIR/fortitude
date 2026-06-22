use crate::{
    CheckContext,
    diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation},
};
use fortitude_macros::ViolationMetadata;
use itertools::Itertools;
use ruff_macros::derive_message_formats;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// ## What it does
/// Checks for the use of tab characters as whitespace
///
/// ## Why is this bad?
/// Tabs are not part of the Fortran standard, and compilers may
/// reject the source if using a strict conformance mode (for example,
/// `gfortran -std=f2023 -Werror`).
///
/// ## Options
/// If the more fine grained option (`check.invalid-tab.indent-width`) is provided this will take precedent.
/// - `check.indent-width`
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

pub(crate) fn check_invalid_tab(context: &CheckContext, root: &Node) -> Vec<Diagnostic> {
    context
        .source_text()
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
            let width = if context.settings().invalid_tab.indent_width.as_usize() == 0usize {
                context.settings().indent_width
            } else {
                context.settings().invalid_tab.indent_width.as_usize()
            };
            let indent = format!("{:width$}", " ");
            let edit = Edit::range_replacement(indent, range);
            context
                .create_diagnostic(InvalidTab, range)
                .with_fix(Fix::unsafe_edit(edit))
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
