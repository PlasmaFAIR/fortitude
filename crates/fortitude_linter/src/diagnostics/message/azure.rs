// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use super::{DiagnosticMessage, Emitter};

/// Generate error logging commands for Azure Pipelines format.
/// See [documentation](https://learn.microsoft.com/en-us/azure/devops/pipelines/scripts/logging-commands?view=azure-devops&tabs=bash#logissue-log-an-error-or-warning)
#[derive(Default)]
pub struct AzureEmitter;

impl Emitter for AzureEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        for message in messages {
            let location = message.compute_start_location();

            writeln!(
                writer,
                "##vso[task.logissue type=error\
                        ;sourcepath={filename};linenumber={line};columnnumber={col};code={code};]{body}",
                filename = message.filename(),
                line = location.row,
                col = location.column,
                code = message.rule().noqa_code(),
                body = message.body(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::diagnostics::message::tests::{capture_emitter_output, create_messages};

    use super::AzureEmitter;

    #[test]
    fn output() {
        let mut emitter = AzureEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
