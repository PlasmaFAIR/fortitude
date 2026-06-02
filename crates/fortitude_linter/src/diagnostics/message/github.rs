// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::{fs::relativize_path, settings::Severity};

use super::Emitter;
use crate::Diagnostic;

/// Generate error workflow command in GitHub Actions format.
/// See: [GitHub documentation](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message)
#[derive(Default)]
pub struct GithubEmitter;

impl Emitter for GithubEmitter {
    fn emit(&mut self, writer: &mut dyn Write, messages: &[Diagnostic]) -> anyhow::Result<()> {
        for message in messages {
            let location = message.compute_start_location();
            let end_location = message.compute_end_location();

            let severity = match message.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info | Severity::None => "notice",
            };

            write!(
                writer,
                "::{severity} title=Fortitude ({code}),file={file},line={row},col={column},endLine={end_row},endColumn={end_column}::",
                code = message.rule().noqa_code(),
                file = message.filename(),
                row = location.line,
                column = location.column,
                end_row = end_location.line,
                end_column = end_location.column,
            )?;

            write!(
                writer,
                "{path}:{row}:{column}:",
                path = relativize_path(message.filename()),
                row = location.line,
                column = location.column,
            )?;

            write!(writer, " {}", message.rule().noqa_code())?;
            writeln!(writer, " {}", message.body())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::GithubEmitter;
    use crate::{
        diagnostics::message::tests::{capture_emitter_output, create_messages},
        settings::Severity,
    };

    #[test]
    fn output() {
        let mut emitter = GithubEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }

    #[test]
    fn output_as_severity_none() {
        let mut emitter = GithubEmitter::default();
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
        let mut emitter = GithubEmitter::default();
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
        let mut emitter = GithubEmitter::default();
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
        let mut emitter = GithubEmitter::default();
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
