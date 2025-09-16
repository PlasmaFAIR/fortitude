//! Access to the Fortitude linting API for the LSP

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{DIAGNOSTIC_NAME, PositionEncoding, edit::ToRangeExt, session::DocumentQuery};
use fortitude_linter::{
    ast_entrypoint_map, check_only_file, diagnostic_message::DiagnosticMessage, locator::Locator,
    rules_to_path_rules, rules_to_text_rules,
};
use ruff_diagnostics::{Applicability, Fix};
use ruff_source_file::{LineIndex, SourceFileBuilder};
use ruff_text_size::{Ranged, TextRange};

/// This is serialized on the diagnostic `data` field.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct AssociatedDiagnosticData {
    /// The message describing what the fix does, if it exists, or the diagnostic name otherwise.
    pub(crate) title: String,
    /// Edits to fix the diagnostic. If this is empty, a fix
    /// does not exist.
    pub(crate) edits: Vec<lsp_types::TextEdit>,
    /// The NOQA code for the diagnostic.
    pub(crate) code: String,
}

/// Describes a fix for `fixed_diagnostic` that may have quick fix
/// edits available, `noqa` comment edits, or both.
#[derive(Clone, Debug)]
pub(crate) struct DiagnosticFix {
    /// The original diagnostic to be fixed
    pub(crate) fixed_diagnostic: lsp_types::Diagnostic,
    /// The message describing what the fix does.
    pub(crate) title: String,
    /// The NOQA code for the diagnostic.
    pub(crate) code: String,
    /// Edits to fix the diagnostic. If this is empty, a fix
    /// does not exist.
    pub(crate) edits: Vec<lsp_types::TextEdit>,
}

/// A series of diagnostics across a single text document
pub(crate) type DiagnosticsMap = FxHashMap<lsp_types::Url, Vec<lsp_types::Diagnostic>>;

pub(crate) fn check(query: &DocumentQuery, encoding: PositionEncoding) -> DiagnosticsMap {
    let source = query.make_source_kind();
    let settings = query.settings();
    let document_path = query.virtual_file_path();

    // TODO(peter): If the document is excluded, return an empty list of diagnostics.

    let file = SourceFileBuilder::new(document_path.to_string_lossy(), source.as_str()).finish();

    let rules = &settings.check.rules;
    let path_rules = rules_to_path_rules(rules);
    let text_rules = rules_to_text_rules(rules);
    let ast_entrypoints = ast_entrypoint_map(rules);

    let diagnostics = check_only_file(
        rules,
        &path_rules,
        &text_rules,
        &ast_entrypoints,
        &document_path,
        &file,
        settings,
        fortitude_linter::settings::IgnoreAllowComments::Disabled,
    )
    .unwrap_or_default();

    let mut diagnostics_map = DiagnosticsMap::default();

    // Populates all relevant URLs with an empty diagnostic list.
    // This ensures that documents without diagnostics still get updated.
    diagnostics_map
        .entry(query.make_key().into_url())
        .or_default();

    let locator = Locator::new(&source);

    let lsp_diagnostics = diagnostics.into_iter().map(|message| {
        to_lsp_diagnostic(&message, file.source_text(), locator.to_index(), encoding)
    });

    diagnostics_map
        .entry(query.make_key().into_url())
        .or_default()
        .extend(lsp_diagnostics);

    diagnostics_map
}

/// Converts LSP diagnostics to a list of `DiagnosticFix`es by deserializing associated data on each diagnostic.
pub(crate) fn fixes_for_diagnostics(
    diagnostics: Vec<lsp_types::Diagnostic>,
) -> crate::Result<Vec<DiagnosticFix>> {
    diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.source.as_deref() == Some(DIAGNOSTIC_NAME))
        .map(move |mut diagnostic| {
            let Some(data) = diagnostic.data.take() else {
                return Ok(None);
            };
            let fixed_diagnostic = diagnostic;
            let associated_data: crate::lint::AssociatedDiagnosticData =
                serde_json::from_value(data).map_err(|err| {
                    anyhow::anyhow!("failed to deserialize diagnostic data: {err}")
                })?;
            Ok(Some(DiagnosticFix {
                fixed_diagnostic,
                code: associated_data.code,
                title: associated_data.title,
                edits: associated_data.edits,
            }))
        })
        .filter_map(crate::Result::transpose)
        .collect()
}

/// Generates an LSP diagnostic with an associated cell index for the diagnostic to go in.
/// If the source kind is a text document, the cell index will always be `0`.
fn to_lsp_diagnostic(
    diagnostic: &DiagnosticMessage,
    source: &str,
    index: &LineIndex,
    encoding: PositionEncoding,
) -> lsp_types::Diagnostic {
    let diagnostic_range = diagnostic.range();
    let name = diagnostic.name();
    let body = diagnostic.body().to_string();
    let fix = diagnostic.fix();
    let suggestion = diagnostic.suggestion();
    let code = diagnostic.code().to_string();

    let fix = fix.and_then(|fix| fix.applies(Applicability::Unsafe).then_some(fix));

    let data = fix
        .is_some()
        .then(|| {
            let edits = fix
                .into_iter()
                .flat_map(Fix::edits)
                .map(|edit| lsp_types::TextEdit {
                    range: diagnostic_edit_range(edit.range(), source, index, encoding),
                    new_text: edit.content().unwrap_or_default().to_string(),
                })
                .collect();
            serde_json::to_value(AssociatedDiagnosticData {
                title: suggestion.unwrap_or(name).to_string(),
                edits,
                code: code.clone(),
            })
            .ok()
        })
        .flatten();

    let range = diagnostic_range.to_range(source, index, encoding);

    let (severity, tags, code) = (
        Some(severity(&code)),
        tags(&code),
        Some(lsp_types::NumberOrString::String(code)),
    );

    lsp_types::Diagnostic {
        range,
        severity,
        tags,
        code,
        code_description: lsp_types::Url::parse(&diagnostic.to_fortitude_url())
            .ok()
            .and_then(|url| Some(lsp_types::CodeDescription { href: url })),
        source: Some(DIAGNOSTIC_NAME.into()),
        message: body,
        related_information: None,
        data,
    }
}

fn diagnostic_edit_range(
    range: TextRange,
    source: &str,
    index: &LineIndex,
    encoding: PositionEncoding,
) -> lsp_types::Range {
    range.to_range(source, index, encoding)
}

fn severity(code: &str) -> lsp_types::DiagnosticSeverity {
    match code {
        // F821: undefined name <name>
        // E902: IOError
        "F821" | "E902" => lsp_types::DiagnosticSeverity::ERROR,
        _ => lsp_types::DiagnosticSeverity::WARNING,
    }
}

fn tags(code: &str) -> Option<Vec<lsp_types::DiagnosticTag>> {
    match code {
        // F401: <module> imported but unused
        // F841: local variable <name> is assigned to but never used
        // RUF059: Unused unpacked variable
        "F401" | "F841" | "RUF059" => Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
        _ => None,
    }
}
