// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use serde_json::json;

use super::Emitter;
use crate::Diagnostic;
use crate::fs::{relativize_path, relativize_path_to};
use crate::settings::Severity;

/// Generate JSON with violations in GitLab CI format
//  https://docs.gitlab.com/ee/ci/testing/code_quality.html#implement-a-custom-tool
pub struct GitlabEmitter {
    project_dir: Option<String>,
}

impl Default for GitlabEmitter {
    fn default() -> Self {
        Self {
            project_dir: std::env::var("CI_PROJECT_DIR").ok(),
        }
    }
}

impl Emitter for GitlabEmitter {
    fn emit(&mut self, writer: &mut dyn Write, messages: &[Diagnostic]) -> anyhow::Result<()> {
        serde_json::to_writer_pretty(
            writer,
            &SerializedMessages {
                messages,
                project_dir: self.project_dir.as_deref(),
            },
        )?;

        Ok(())
    }
}

struct SerializedMessages<'a> {
    messages: &'a [Diagnostic],
    project_dir: Option<&'a str>,
}

impl Serialize for SerializedMessages<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.messages.len()))?;
        let mut fingerprints = HashSet::<u64>::with_capacity(self.messages.len());

        for message in self.messages {
            let start_location = message.compute_start_location();
            let end_location = message.compute_end_location();

            let lines = json!({
                "begin": start_location.line,
                "end": end_location.line
            });

            let path = self.project_dir.as_ref().map_or_else(
                || relativize_path(message.filename()),
                |project_dir| relativize_path_to(message.filename(), project_dir),
            );

            let mut message_fingerprint = fingerprint(message, &path, 0);

            // Make sure that we do not get a fingerprint that is already in use
            // by adding in the previously generated one.
            while fingerprints.contains(&message_fingerprint) {
                message_fingerprint = fingerprint(message, &path, message_fingerprint);
            }
            fingerprints.insert(message_fingerprint);

            let description = format!("({}) {}", message.rule().noqa_code(), message.body());
            let name = format!(
                "{}: {}",
                message.rule().noqa_code(),
                message.rule().as_ref()
            );

            let severity = match message.severity {
                Severity::Error => "critical",
                Severity::Warning => "major",
                Severity::Info | Severity::None => "info",
            };

            let value = json!({
                "description": description,
                "check_name": name,
                "severity": severity,
                "fingerprint": format!("{:x}", message_fingerprint),
                "location": {
                    "path": path,
                    "lines": lines
                }
            });

            s.serialize_element(&value)?;
        }

        s.end()
    }
}

/// Generate a unique fingerprint to identify a violation.
fn fingerprint(message: &Diagnostic, project_path: &str, salt: u64) -> u64 {
    let mut hasher = DefaultHasher::new();

    salt.hash(&mut hasher);
    message.name().hash(&mut hasher);
    project_path.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::GitlabEmitter;
    use crate::{
        diagnostics::message::tests::{capture_emitter_output, create_messages},
        settings::Severity,
    };

    #[test]
    fn output() {
        let mut emitter = GitlabEmitter::default();
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(redact_fingerprint(&content));
    }

    #[test]
    fn output_as_severity_none() {
        let mut emitter = GitlabEmitter::default();
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::None;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(redact_fingerprint(&content));
    }

    #[test]
    fn output_as_severity_info() {
        let mut emitter = GitlabEmitter::default();
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Info;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(redact_fingerprint(&content));
    }

    #[test]
    fn output_as_severity_warning() {
        let mut emitter = GitlabEmitter::default();
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Warning;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(redact_fingerprint(&content));
    }

    #[test]
    fn output_as_severity_error() {
        let mut emitter = GitlabEmitter::default();
        let messages = create_messages()
            .iter_mut()
            .map(|message| {
                message.severity = Severity::Error;
                message.clone()
            })
            .collect::<Vec<_>>();

        let content = capture_emitter_output(&mut emitter, &messages);

        assert_snapshot!(redact_fingerprint(&content));
    }

    // Redact the fingerprint because the default hasher isn't stable across platforms.
    fn redact_fingerprint(content: &str) -> String {
        static FINGERPRINT_HAY_KEY: &str = r#""fingerprint": ""#;

        let mut output = String::with_capacity(content.len());
        let mut last = 0;

        for (start, _) in content.match_indices(FINGERPRINT_HAY_KEY) {
            let fingerprint_hash_start = start + FINGERPRINT_HAY_KEY.len();
            output.push_str(&content[last..fingerprint_hash_start]);
            output.push_str("<redacted>");
            last = fingerprint_hash_start
                + content[fingerprint_hash_start..]
                    .find('"')
                    .expect("Expected terminating quote");
        }

        output.push_str(&content[last..]);

        output
    }
}
