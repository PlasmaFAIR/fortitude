// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::io::Write;

use crate::fs::relativize_path;
use crate::message::Emitter;

use super::DiagnosticMessage;

/// Generate error workflow command in GitHub Actions format.
/// See: [GitHub documentation](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message)
#[derive(Default)]
pub struct GithubEmitter;

impl Emitter for GithubEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        for message in messages {
            let source_location = message.compute_start_location();
            let location = source_location.clone();

            let end_location = message.compute_end_location();

            write!(
                writer,
                "::error title=Fortitude{code},file={file},line={row},col={column},endLine={end_row},endColumn={end_column}::",
                code = message.rule().map_or_else(String::new, |rule| format!(" ({})", rule.noqa_code())),
                file = message.filename(),
                row = source_location.row,
                column = source_location.column,
                end_row = end_location.row,
                end_column = end_location.column,
            )?;

            write!(
                writer,
                "{path}:{row}:{column}:",
                path = relativize_path(message.filename()),
                row = location.row,
                column = location.column,
            )?;

            if let Some(rule) = message.rule() {
                write!(writer, " {}", rule.noqa_code())?;
            }

            writeln!(writer, " {}", message.body())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::GithubEmitter;

    #[test]
    fn output() {
        let mut emitter = GithubEmitter;
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
