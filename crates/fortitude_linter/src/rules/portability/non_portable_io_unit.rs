use crate::ast::FortitudeNode;
use crate::rules::utilities::literal_as_io_unit;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for certain literals as units in `read`/`write` statements.
///
/// ## Why is this bad?
/// The Fortran standard does not specify numeric values for `stdin`, `stdout`, or
/// `stderr`, and although many compilers do "pre-connect" units `5`, `6`, and `0`,
/// respectively, some use other numbers. Instead, use the named constants `input_unit`,
/// `output_unit`, or `error_unit` from the `iso_fortran_env` module.
///
/// !!! note
///     An `open` statement with one of these units is completely portable, it is just
///     the use to mean `stdin`/`stdout`/`stderr` without an explicit `open` that is
///     non-portable -- but see also [`magic-io-unit`](magic-io-unit.md) for why it's
///     best to avoid literal integers as IO units altogether.
///
/// ## Options
/// - `check.portability.allow-cray-file-units`
#[derive(ViolationMetadata)]
pub(crate) struct NonPortableIoUnit {
    value: i32,
    kind: String,
    replacement: String,
}

impl Violation for NonPortableIoUnit {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { value, kind, .. } = self;
        format!("Non-portable unit '{value}' in '{kind}' statement")
    }

    fn fix_title(&self) -> Option<String> {
        let Self { replacement, .. } = self;
        Some(format!("Use `{replacement}` from `iso_fortran_env`"))
    }
}

impl AstRule for NonPortableIoUnit {
    fn check(
        settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let unit = literal_as_io_unit(node, src)?;

        let value = unit
            .to_text(src.source_text())?
            .parse::<i32>()
            .unwrap_or_default();
        let is_read = node.kind() == "read_statement";
        let is_write = node.kind() == "write_statement";

        let kind = if is_read { "read" } else { "write" }.to_string();

        let mut stdin = vec![5];
        let mut stdout = vec![6];
        let mut stderr = vec![0];

        if !settings.portability.allow_cray_file_units {
            stdin.push(100);
            stdout.push(101);
            stderr.push(102);
        }

        let replacement = if is_read && stdin.contains(&value) {
            "input_unit"
        } else if is_write && stdout.contains(&value) {
            "output_unit"
        } else if is_write && stderr.contains(&value) {
            "error_unit"
        } else {
            return None;
        }
        .to_string();

        some_vec!(Diagnostic::from_node(
            Self {
                value,
                kind,
                replacement
            },
            &unit
        ))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["read_statement", "write_statement"]
    }
}
