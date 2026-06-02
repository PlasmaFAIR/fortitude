// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use super::Emitter;
use crate::{Diagnostic, settings::Severity};

/// Generate error logging commands for Azure Pipelines format.
/// See [documentation](https://learn.microsoft.com/en-us/azure/devops/pipelines/scripts/logging-commands?view=azure-devops&tabs=bash#logissue-log-an-error-or-warning)
#[derive(Default)]
pub struct AzureEmitter;

impl Emitter for AzureEmitter {
    fn emit(&mut self, writer: &mut dyn Write, messages: &[Diagnostic]) -> anyhow::Result<()> {
        for message in messages {
            let location = message.compute_start_location();

            let severity = match message.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info | Severity::None => "debug",
            };

            writeln!(
                writer,
                "##vso[task.logissue type={severity}\
                        ;sourcepath={filename};linenumber={line};columnnumber={col};code={code};]{body}",
                filename = message.filename(),
                line = location.line,
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

    use crate::{
        diagnostics::message::tests::{capture_emitter_output, create_messages},
        settings::Severity,
    };

    use super::AzureEmitter;

    #[test]
    fn output() {
        let mut emitter = AzureEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }

    #[test]
    fn output_as_severity_none() {
        let mut emitter = AzureEmitter::default();
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
        let mut emitter = AzureEmitter::default();
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
        let mut emitter = AzureEmitter::default();
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
        let mut emitter = AzureEmitter::default();
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
}
