// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use super::{DiagnosticMessage, Emitter};
use crate::fs::relativize_path;

/// Generate violations in Pylint format.
/// See: [Flake8 documentation](https://flake8.pycqa.org/en/latest/internal/formatters.html#pylint-formatter)
#[derive(Default)]
pub struct PylintEmitter;

impl Emitter for PylintEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        for message in messages {
            let row = message.compute_start_location().row;

            let body = format!(
                "[{code}] {body}",
                code = message.rule().noqa_code(),
                body = message.body()
            );

            writeln!(
                writer,
                "{path}:{row}: {body}",
                path = relativize_path(message.filename()),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::PylintEmitter;
    use crate::diagnostics::message::tests::{capture_emitter_output, create_messages};

    #[test]
    fn output() {
        let mut emitter = PylintEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
