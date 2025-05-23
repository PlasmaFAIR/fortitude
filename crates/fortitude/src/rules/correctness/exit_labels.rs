use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
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
#[derive(ViolationMetadata)]
pub(crate) struct MissingExitOrCycleLabel {
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
                let edit = Edit::insertion(label_with_space, stmt.end_textsize());
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

/// ## What does it do?
/// Checks for `exit` or `cycle` in unnamed `do` loops
///
/// ## Why is this bad?
/// Using loop labels with `exit` and `cycle` statements prevents bugs when exiting the
/// wrong loop, and helps readability in deeply nested or long loops. The danger is
/// particularly enhanced when code is refactored to add further loops.
///
/// ## Settings
/// See [allow-unnested-loops](../settings.md#check_exit-unlabelled-loops_allow-unnested-loops)
#[derive(ViolationMetadata)]
pub(crate) struct ExitOrCycleInUnlabelledLoop {
    name: String,
}

impl Violation for ExitOrCycleInUnlabelledLoop {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { name } = self;
        format!("'{name}' statement in unlabelled 'do' loop")
    }
}

impl AstRule for ExitOrCycleInUnlabelledLoop {
    fn check(settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>> {
        let src = source.source_text();
        let name = node.to_text(src)?.to_lowercase();
        // This filters to the keywords we want that _also_ don't have a label
        if !matches!(name.as_str(), "exit" | "cycle") {
            return None;
        }

        let parent_loop = node
            .ancestors()
            .filter(|ancestor| ancestor.kind() == "do_loop_statement")
            .nth(0)?;

        // Immediate parent loop has a label, but we don't want to warn here, because
        // that's covered by missing-exit-or-cycle-label
        if parent_loop
            .child_with_name("block_label_start_expression")
            .is_some()
        {
            return None;
        }

        // If we're only supposed to check on nested loops, check that there is at least
        // one more level of nesting
        if settings.check.exit_unlabelled_loops.allow_unnested_loops {
            parent_loop
                .ancestors()
                .filter(|ancestor| ancestor.kind() == "do_loop_statement")
                .nth(0)?;
        }

        some_vec!(Diagnostic::from_node(Self { name }, node))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["keyword_statement"]
    }
}

/// ## What does it do?
/// Check for missing block labels on `end` statements
///
/// ## Why is this bad?
/// This is a compile error!
#[derive(ViolationMetadata)]
pub(crate) struct MissingEndLabel {
    start_name: String,
    end_name: String,
    label: String,
}

impl AlwaysFixableViolation for MissingEndLabel {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self {
            start_name,
            end_name,
            ..
        } = self;
        format!("'{end_name}' statement in named '{start_name}' block missing label")
    }

    fn fix_title(&self) -> String {
        let Self { label, .. } = self;
        format!("Add label '{label}'")
    }
}

impl AstRule for MissingEndLabel {
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

        let end = match node.kind() {
            "select_case_statement" | "select_type_statement" | "select_rank_statement" => {
                "end_select_statement".to_string()
            }
            _ => format!("end_{}", node.kind()),
        };

        let violation: Diagnostic = node
            .child_with_name(&end)
            .filter(|node| node.child_with_name("block_label").is_none())
            .map(|stmt| (stmt, stmt.to_text(src).unwrap_or_default().to_lowercase()))
            .map(|(stmt, end_name)| {
                let label_with_space = format!(" {label}");
                let edit = Edit::insertion(label_with_space, stmt.end_textsize());
                let fix = Fix::safe_edit(edit);

                let start_name = node.child(1).unwrap().to_text(src).unwrap().to_lowercase();

                Diagnostic::from_node(
                    Self {
                        end_name: end_name.to_string(),
                        start_name,
                        label: label.to_string(),
                    },
                    &stmt,
                )
                .with_fix(fix)
            })?;

        Some(vec![violation])
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![
            "associate_statement",
            "block_construct",
            "coarray_critical_statement",
            "coarray_team_statement",
            "do_loop_statement",
            "forall_statement",
            "if_statement",
            "select_case_statement",
            "select_rank_statement",
            "select_type_statement",
            "where_statement",
        ]
    }
}

pub(crate) mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub allow_unnested_loops: bool,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.exit_unlabelled_loops",
                fields = [self.allow_unnested_loops]
            }
            Ok(())
        }
    }
}
