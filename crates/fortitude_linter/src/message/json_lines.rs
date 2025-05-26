// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::message::json::message_to_json_value;
use crate::message::Emitter;

use super::DiagnosticMessage;

#[derive(Default)]
pub struct JsonLinesEmitter;

impl Emitter for JsonLinesEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        for message in messages {
            serde_json::to_writer(&mut *writer, &message_to_json_value(message))?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::json_lines::JsonLinesEmitter;
    use crate::message::tests::{capture_emitter_output, create_messages};

    #[test]
    fn output() {
        let mut emitter = JsonLinesEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
