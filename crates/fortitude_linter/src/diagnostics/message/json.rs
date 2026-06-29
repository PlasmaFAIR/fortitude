// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer, ser::SerializeSeq};
use serde_json::{Value, json};

use ruff_diagnostics::{Applicability, Edit};
use ruff_source_file::{LineColumn, OneIndexed, SourceFile};
use ruff_text_size::Ranged;

use crate::diagnostics::{ConciseMessage, Diagnostic, DisplayDiagnosticConfig, Severity};

pub(super) struct JsonRenderer<'a> {
    config: &'a DisplayDiagnosticConfig,
}

impl<'a> JsonRenderer<'a> {
    pub(super) fn new(config: &'a DisplayDiagnosticConfig) -> Self {
        Self { config }
    }
}

impl JsonRenderer<'_> {
    pub(super) fn render(
        &self,
        f: &mut std::fmt::Formatter,
        diagnostics: &[Diagnostic],
    ) -> std::fmt::Result {
        write!(
            f,
            "{:#}",
            diagnostics_to_json_value(diagnostics, self.config)
        )
    }
}

fn diagnostics_to_json_value<'a>(
    diagnostics: impl IntoIterator<Item = &'a Diagnostic>,
    config: &DisplayDiagnosticConfig,
) -> Value {
    let values: Vec<_> = diagnostics
        .into_iter()
        .map(|diag| diagnostic_to_json(diag, config))
        .collect();
    json!(values)
}

pub(super) fn diagnostic_to_json<'a>(
    diagnostic: &'a Diagnostic,
    config: &'a DisplayDiagnosticConfig,
) -> JsonDiagnostic<'a> {
    let span = diagnostic.primary_span_ref();
    let filename = span.map(|span| span.file().name());
    let range = span.and_then(|span| span.range());
    let diagnostic_source = span.map(|span| span.file().clone());
    let source_code = diagnostic_source
        .as_ref()
        .map(|diagnostic_source| diagnostic_source.to_source_code());

    let mut start_location = None;
    let mut end_location = None;
    if let Some(source_code) = source_code
        && let Some(range) = range
    {
        let start = source_code.line_column(range.start());
        let end = source_code.line_column(range.end());
        start_location = Some(start);
        end_location = Some(end);
    }

    let fix = diagnostic.fix().map(|fix| JsonFix {
        applicability: fix.applicability(),
        message: diagnostic.first_help_text(),
        edits: ExpandedEdits {
            edits: fix.edits(),
            config,
            diagnostic_source,
        },
    });

    // In preview, the locations and filename can be optional
    // and the severity is displayed.
    if config.preview {
        JsonDiagnostic {
            code: diagnostic.secondary_code_or_id(),
            severity: diagnostic.severity(),
            url: diagnostic.documentation_url(),
            message: diagnostic.concise_message(),
            fix,
            location: start_location.map(JsonLocation::from),
            end_location: end_location.map(JsonLocation::from),
            filename,
        }
    } else {
        JsonDiagnostic {
            code: diagnostic.secondary_code_or_id(),
            severity: Severity::Error,
            url: diagnostic.documentation_url(),
            message: diagnostic.concise_message(),
            fix,
            location: Some(start_location.unwrap_or_default().into()),
            end_location: Some(end_location.unwrap_or_default().into()),
            filename: Some(filename.unwrap_or_default()),
        }
    }
}

struct ExpandedEdits<'a> {
    edits: &'a [Edit],
    config: &'a DisplayDiagnosticConfig,
    diagnostic_source: Option<SourceFile>,
}

impl Serialize for ExpandedEdits<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.edits.len()))?;

        for edit in self.edits {
            let (location, end_location) = if let Some(diagnostic_source) = &self.diagnostic_source
            {
                let source_code = diagnostic_source.to_source_code();
                let location = source_code.line_column(edit.start());
                let end_location = source_code.line_column(edit.end());

                (Some(location), Some(end_location))
            } else {
                (None, None)
            };

            // In preview, the locations can be optional.
            let value = if self.config.preview {
                JsonEdit {
                    content: edit.content().unwrap_or_default(),
                    location: location.map(JsonLocation::from),
                    end_location: end_location.map(JsonLocation::from),
                }
            } else {
                JsonEdit {
                    content: edit.content().unwrap_or_default(),
                    location: Some(location.unwrap_or_default().into()),
                    end_location: Some(end_location.unwrap_or_default().into()),
                }
            };

            s.serialize_element(&value)?;
        }

        s.end()
    }
}

/// A serializable version of `Diagnostic`.
#[derive(Serialize)]
pub(crate) struct JsonDiagnostic<'a> {
    code: &'a str,
    severity: Severity,
    end_location: Option<JsonLocation>,
    filename: Option<&'a str>,
    fix: Option<JsonFix<'a>>,
    location: Option<JsonLocation>,
    message: ConciseMessage<'a>,
    url: Option<&'a str>,
}

#[derive(Serialize)]
struct JsonFix<'a> {
    applicability: Applicability,
    edits: ExpandedEdits<'a>,
    message: Option<&'a str>,
}

#[derive(Serialize)]
struct JsonLocation {
    column: OneIndexed,
    row: OneIndexed,
}

impl From<LineColumn> for JsonLocation {
    fn from(location: LineColumn) -> Self {
        JsonLocation {
            row: location.line,
            column: location.column,
        }
    }
}

#[derive(Serialize)]
struct JsonEdit<'a> {
    content: &'a str,
    end_location: Option<JsonLocation>,
    location: Option<JsonLocation>,
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{
        OutputFormat,
        message::tests::{TestEnvironment, create_diagnostics, create_syntax_error_diagnostics},
    };

    #[test]
    fn output() {
        let (env, diagnostics) = create_diagnostics(OutputFormat::Json);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn syntax_errors() {
        let (env, diagnostics) = create_syntax_error_diagnostics(OutputFormat::Json);
        insta::assert_snapshot!(env.render_diagnostics(&diagnostics));
    }

    #[test]
    fn missing_file_stable() {
        let mut env = TestEnvironment::new();
        env.format(OutputFormat::Json);
        env.preview(false);

        let diag = env
            .err()
            .documentation_url("https://docs.astral.sh/ruff/rules/test-diagnostic")
            .build();

        insta::assert_snapshot!(
            env.render(&diag),
            @r#"
        [
          {
            "code": "stable-test-rule",
            "severity": "error",
            "end_location": {
              "column": 1,
              "row": 1
            },
            "filename": "",
            "fix": null,
            "location": {
              "column": 1,
              "row": 1
            },
            "message": "main diagnostic message",
            "url": "https://docs.astral.sh/ruff/rules/test-diagnostic"
          }
        ]
        "#,
        );
    }

    #[test]
    fn missing_file_preview() {
        let mut env = TestEnvironment::new();
        env.format(OutputFormat::Json);
        env.preview(true);

        let diag = env
            .err()
            .documentation_url("https://docs.astral.sh/ruff/rules/test-diagnostic")
            .build();

        insta::assert_snapshot!(
            env.render(&diag),
            @r#"
        [
          {
            "code": "stable-test-rule",
            "severity": "error",
            "end_location": null,
            "filename": null,
            "fix": null,
            "location": null,
            "message": "main diagnostic message",
            "url": "https://docs.astral.sh/ruff/rules/test-diagnostic"
          }
        ]
        "#,
        );
    }
}
