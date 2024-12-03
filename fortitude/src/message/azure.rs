// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::message::Emitter;

use super::DiagnosticMessage;

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
                        ;sourcepath={filename};linenumber={line};columnnumber={col};{code}]{body}",
                filename = message.filename(),
                line = location.row,
                col = location.column,
                code = message
                    .rule()
                    .map_or_else(String::new, |rule| format!("code={};", rule.noqa_code())),
                body = message.body(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::AzureEmitter;

    #[test]
    fn output() {
        let mut emitter = AzureEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
