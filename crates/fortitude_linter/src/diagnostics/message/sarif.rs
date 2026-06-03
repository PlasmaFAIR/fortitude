// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::collections::HashSet;
use std::io::Write;

use anyhow::Result;
use serde::{Serialize, Serializer};
use serde_json::json;

use ruff_source_file::OneIndexed;

use crate::VERSION;
use crate::fs::normalize_path;
use crate::registry::{Category, RuleNamespace};
use crate::rules::Rule;
use crate::settings::Severity;

use super::Emitter;
use crate::Diagnostic;

pub struct SarifEmitter;

impl Emitter for SarifEmitter {
    fn emit(&mut self, writer: &mut dyn Write, messages: &[Diagnostic]) -> Result<()> {
        let results = messages
            .iter()
            .map(SarifResult::from_message)
            .collect::<Result<Vec<_>>>()?;

        let unique_rules: HashSet<_> = results.iter().map(|result| result.rule).collect();
        let mut rules: Vec<SarifRule> = unique_rules.into_iter().map(SarifRule::from).collect();
        rules.sort_by(|a, b| a.code.cmp(&b.code));

        let output = json!({
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "fortitude",
                        "informationUri": "https://github.com/PlasmaFAIR/fortitude",
                        "rules": rules,
                        "version": VERSION.to_string(),
                    }
                },
                "results": results,
            }],
        });
        serde_json::to_writer_pretty(writer, &output)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SarifRule<'a> {
    name: &'a str,
    code: String,
    linter: &'a str,
    summary: &'a str,
    explanation: Option<&'a str>,
    level: &'a str,
    // url: Option<String>,
}

impl From<Rule> for SarifRule<'_> {
    fn from(rule: Rule) -> Self {
        let code = rule.noqa_code().to_string();
        let (linter, _) = Category::parse_code(&code).unwrap();
        let level = match rule.severity() {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info | Severity::None => "note",
        };
        Self {
            name: rule.into(),
            code,
            linter: linter.name(),
            summary: rule.message_formats()[0],
            explanation: rule.explanation(),
            level,
            // url: rule.url(),
        }
    }
}

impl Serialize for SarifRule<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        json!({
            "id": self.code,
            "shortDescription": {
                "text": self.summary,
            },
            "fullDescription": {
                "text": self.explanation,
            },
            "help": {
                "text": self.summary,
            },
            // "helpUri": self.url,
            "properties": {
                "id": self.code,
                "kind": self.linter,
                "name": self.name,
                "problem.severity": self.level,
            },
        })
        .serialize(serializer)
    }
}

#[derive(Debug)]
struct SarifResult {
    rule: Rule,
    level: String,
    message: String,
    uri: String,
    start_line: OneIndexed,
    start_column: OneIndexed,
    end_line: OneIndexed,
    end_column: OneIndexed,
}

impl SarifResult {
    #[cfg(not(target_arch = "wasm32"))]
    fn from_message(message: &Diagnostic) -> Result<Self> {
        use crate::settings::Severity;

        let start_location = message.compute_start_location();
        let end_location = message.compute_end_location();
        let path = normalize_path(message.filename());
        let level = match message.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info | Severity::None => "note",
        };
        Ok(Self {
            rule: message.rule(),
            level: level.to_string(),
            message: message.body().to_string(),
            uri: url::Url::from_file_path(&path)
                .map_err(|()| anyhow::anyhow!("Failed to convert path to URL: {}", path.display()))?
                .to_string(),
            start_line: start_location.line,
            start_column: start_location.column,
            end_line: end_location.line,
            end_column: end_location.column,
        })
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::unnecessary_wraps)]
    fn from_message(message: &Message) -> Result<Self> {
        let start_location = message.compute_start_location();
        let end_location = message.compute_end_location();
        let path = normalize_path(message.filename());
        Ok(Self {
            rule: message.rule(),
            level: "error".to_string(),
            message: message.body().to_string(),
            uri: path.display().to_string(),
            start_line: start_location.line,
            start_column: start_location.column,
            end_line: end_location.line,
            end_column: end_location.column,
        })
    }
}

impl Serialize for SarifResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        json!({
            "level": self.level,
            "message": {
                "text": self.message,
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": self.uri,
                    },
                    "region": {
                        "startLine": self.start_line,
                        "startColumn": self.start_column,
                        "endLine": self.end_line,
                        "endColumn": self.end_column,
                    }
                }
            }],
            "ruleId": self.rule.noqa_code().to_string(),
        })
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::SarifEmitter;
    use crate::{
        diagnostics::message::tests::{capture_emitter_output, create_messages},
        settings::Severity,
    };

    fn get_output() -> String {
        let mut emitter = SarifEmitter {};
        capture_emitter_output(&mut emitter, &create_messages())
    }

    #[test]
    fn output_as_severity_none() {
        let mut emitter = SarifEmitter {};
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::None;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(content);
    }

    #[test]
    fn output_as_severity_info() {
        let mut emitter = SarifEmitter {};
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Info;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(content);
    }

    #[test]
    fn output_as_severity_warning() {
        let mut emitter = SarifEmitter {};
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Warning;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(content);
    }

    #[test]
    fn output_as_severity_error() {
        let mut emitter = SarifEmitter {};
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Error;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(content);
    }
    #[test]
    fn valid_json() {
        let content = get_output();
        serde_json::from_str::<serde_json::Value>(&content).unwrap();
    }

    #[test]
    fn test_results() {
        let content = get_output();
        let value = serde_json::from_str::<serde_json::Value>(&content).unwrap();

        insta::assert_json_snapshot!(value, {
            ".runs[0].tool.driver.version" => "[VERSION]",
            ".runs[0].results[].locations[].physicalLocation.artifactLocation.uri" => "[URI]",
        });
    }
}
