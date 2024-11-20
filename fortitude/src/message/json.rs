// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use serde_json::{json, Value};

use ruff_diagnostics::Edit;
use ruff_source_file::SourceCode;
use ruff_text_size::Ranged;

use crate::message::{DiagnosticMessage, Emitter};

#[derive(Default)]
pub struct JsonEmitter;

impl Emitter for JsonEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        serde_json::to_writer_pretty(writer, &ExpandedMessages { messages })?;

        Ok(())
    }
}

struct ExpandedMessages<'a> {
    messages: &'a [DiagnosticMessage],
}

impl Serialize for ExpandedMessages<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.messages.len()))?;

        for message in self.messages {
            let value = message_to_json_value(message);
            s.serialize_element(&value)?;
        }

        s.end()
    }
}

pub(crate) fn message_to_json_value(message: &DiagnosticMessage) -> Value {
    let source_code = message.source_file().to_source_code();

    let fix = message.fix().map(|fix| {
        json!({
            "applicability": fix.applicability(),
            "message": message.suggestion(),
            "edits": &ExpandedEdits { edits: fix.edits(), source_code: &source_code },
        })
    });

    let start_location = source_code.source_location(message.start());
    let end_location = source_code.source_location(message.end());

    json!({
        "code": message.rule().map(|rule| rule.noqa_code().to_string()),
        "message": message.body(),
        "fix": fix,
        "location": start_location,
        "end_location": end_location,
        "filename": message.filename(),
    })
}

struct ExpandedEdits<'a> {
    edits: &'a [Edit],
    source_code: &'a SourceCode<'a, 'a>,
}

impl Serialize for ExpandedEdits<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.edits.len()))?;

        for edit in self.edits {
            let location = self.source_code.source_location(edit.start());
            let end_location = self.source_code.source_location(edit.end());

            let value = json!({
                "content": edit.content().unwrap_or_default(),
                "location": location,
                "end_location": end_location
            });

            s.serialize_element(&value)?;
        }

        s.end()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::JsonEmitter;

    #[test]
    fn output() {
        let mut emitter = JsonEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
