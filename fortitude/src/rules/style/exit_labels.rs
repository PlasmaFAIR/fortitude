use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What does it do?
/// When using `exit` or `cycle` in a named `do` loop, the `exit`/`cycle` statement
/// should use the loop name
///
/// ## Example
/// ```f90
/// name: do
///   exit name
/// end do name
/// ```
///
/// Using named loops is particularly useful for nested or complicated loops, as it
/// helps the reader keep track of the flow of logic. It's also the only way to `exit`
/// or `cycle` outer loops from within inner ones.
#[violation]
pub struct MissingExitOrCycleLabel {
    name: String,
    label: String,
}

impl Violation for MissingExitOrCycleLabel {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name, label } = self;
        format!("'{name}' statement in named 'do' loop missing label '{label}'")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { label, .. } = self;
        Some(format!("Add label '{label}'"))
    }
}
impl AstRule for MissingExitOrCycleLabel {
    fn check<'a>(
        _settings: &Settings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {
        let src = src.source_text();
        // Skip unlabelled loops
        let label = node
            .child_with_name("block_label_start_expression")?
            .to_text(src)?
            .trim_end_matches(':');

        let violations: Vec<Diagnostic> = node
            .named_descendants_except(["do_loop_statement"])
            .filter(|node| node.kind() == "keyword_statement")
            .map(|stmt| (stmt, stmt.to_text(src).unwrap_or_default().to_lowercase()))
            .filter(|(_, name)| name == "exit" || name == "cycle")
            .map(|(stmt, name)| {
                let label_with_space = format!(" {label}");
                let edit = Edit::insertion(
                    label_with_space,
                    TextSize::try_from(stmt.end_byte()).unwrap(),
                );
                let fix = Fix::unsafe_edit(edit);

                Diagnostic::from_node(
                    Self {
                        name: name.to_string(),
                        label: label.to_string(),
                    },
                    &stmt,
                )
                .with_fix(fix)
            })
            .collect();

        Some(violations)
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["do_loop_statement"]
    }
}
