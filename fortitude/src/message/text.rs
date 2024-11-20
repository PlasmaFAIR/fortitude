// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::fmt::{Display, Formatter};
use std::io::Write;

use annotate_snippets::{Level, Renderer, Snippet};
use bitflags::bitflags;
use colored::Colorize;

use ruff_source_file::OneIndexed;
use ruff_text_size::{Ranged, TextRange};

use crate::fs::relativize_path;
// use crate::line_width::{IndentWidth, LineWidthBuilder};
// use crate::message::diff::Diff;
use crate::message::Emitter;
use crate::settings::UnsafeFixes;
use crate::text_helpers::ShowNonprinting;

use super::DiagnosticMessage;

bitflags! {
    #[derive(Default)]
    struct EmitterFlags: u8 {
        /// Whether to show the fix status of a diagnostic.
        const SHOW_FIX_STATUS    = 0b0000_0001;
        /// Whether to show the diff of a fix, for diagnostics that have a fix.
        const SHOW_FIX_DIFF      = 0b0000_0010;
        /// Whether to show the source code of a diagnostic.
        const SHOW_SOURCE        = 0b0000_0100;
    }
}

#[derive(Default)]
pub struct TextEmitter {
    flags: EmitterFlags,
    unsafe_fixes: UnsafeFixes,
}

impl TextEmitter {
    #[must_use]
    pub fn with_show_fix_status(mut self, show_fix_status: bool) -> Self {
        self.flags
            .set(EmitterFlags::SHOW_FIX_STATUS, show_fix_status);
        self
    }

    #[must_use]
    pub fn with_show_fix_diff(mut self, show_fix_diff: bool) -> Self {
        self.flags.set(EmitterFlags::SHOW_FIX_DIFF, show_fix_diff);
        self
    }

    #[must_use]
    pub fn with_show_source(mut self, show_source: bool) -> Self {
        self.flags.set(EmitterFlags::SHOW_SOURCE, show_source);
        self
    }

    #[must_use]
    pub fn with_unsafe_fixes(mut self, unsafe_fixes: UnsafeFixes) -> Self {
        self.unsafe_fixes = unsafe_fixes;
        self
    }
}

impl Emitter for TextEmitter {
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()> {
        for message in messages {
            write!(
                writer,
                "{path}{sep}",
                path = relativize_path(message.filename()).bold(),
                sep = ":".cyan(),
            )?;

            let start_location = message.compute_start_location();

            let diagnostic_location = start_location;

            writeln!(
                writer,
                "{row}{sep}{col}{sep} {code_and_body}",
                row = diagnostic_location.row,
                col = diagnostic_location.column,
                sep = ":".cyan(),
                code_and_body = RuleCodeAndBody {
                    message,
                    show_fix_status: self.flags.intersects(EmitterFlags::SHOW_FIX_STATUS),
                    unsafe_fixes: self.unsafe_fixes,
                }
            )?;

            if self.flags.intersects(EmitterFlags::SHOW_SOURCE) {
                // The `0..0` range is used to highlight file-level diagnostics.
                if message.range() != TextRange::default() {
                    writeln!(writer, "{}", MessageCodeFrame { message })?;
                }
            }

            // if self.flags.intersects(EmitterFlags::SHOW_FIX_DIFF) {
            //     if let Some(diff) = Diff::from_message(message) {
            //         writeln!(writer, "{diff}")?;
            //     }
            // }
        }

        Ok(())
    }
}

pub(super) struct RuleCodeAndBody<'a> {
    pub(crate) message: &'a DiagnosticMessage,
    pub(crate) show_fix_status: bool,
    pub(crate) unsafe_fixes: UnsafeFixes,
}

impl Display for RuleCodeAndBody<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.show_fix_status {
            if let Some(fix) = self.message.fix() {
                // Do not display an indicator for inapplicable fixes
                if fix.applies(self.unsafe_fixes.required_applicability()) {
                    if let Some(rule) = self.message.rule() {
                        write!(f, "{} ", rule.noqa_code().to_string().red().bold())?;
                    }
                    return write!(
                        f,
                        "{fix}{body}",
                        fix = format_args!("[{}] ", "*".cyan()),
                        body = self.message.body(),
                    );
                }
            }
        };

        if let Some(rule) = self.message.rule() {
            write!(
                f,
                "{code} {body}",
                code = rule.noqa_code().to_string().red().bold(),
                body = self.message.body(),
            )
        } else {
            f.write_str(self.message.body())
        }
    }
}

pub(super) struct MessageCodeFrame<'a> {
    pub(crate) message: &'a DiagnosticMessage,
}

impl Display for MessageCodeFrame<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source_code = self.message.source_file().to_source_code();

        let content_start_index = source_code.line_index(self.message.start());
        let mut start_index = content_start_index.saturating_sub(2);

        // Trim leading empty lines.
        while start_index < content_start_index {
            if !source_code.line_text(start_index).trim().is_empty() {
                break;
            }
            start_index = start_index.saturating_add(1);
        }

        let content_end_index = source_code.line_index(self.message.end());
        let mut end_index = content_end_index
            .saturating_add(2)
            .min(OneIndexed::from_zero_indexed(source_code.line_count()));

        // Trim trailing empty lines.
        while end_index > content_end_index {
            if !source_code.line_text(end_index).trim().is_empty() {
                break;
            }

            end_index = end_index.saturating_sub(1);
        }

        let start_offset = source_code.line_start(start_index);
        let end_offset = source_code.line_end(end_index);

        let source = source_code.slice(TextRange::new(start_offset, end_offset));
        let message_range = self.message.range() - start_offset;

        let start_char = source[TextRange::up_to(message_range.start())]
            .chars()
            .count();
        let end_char = source[TextRange::up_to(message_range.end())]
            .chars()
            .count();

        let mut code = self.message.code.bold().bright_red();

        // Disable colours for tests, if the user requests it via env var, or non-tty
        if cfg!(test) || !colored::control::SHOULD_COLORIZE.should_colorize() {
            code = code.clear();
        };

        let source_text = source.show_nonprinting();

        let snippet = Level::None.title("").snippet(
            Snippet::source(&source_text)
                .line_start(start_index.get())
                .annotation(Level::Error.span(start_char..end_char).label(&code)),
        );

        let snippet_with_footer = if let Some(s) = self.message.suggestion() {
            snippet.footer(Level::Help.title(s))
        } else {
            snippet
        };

        // Disable colours for tests, if the user requests it via env var, or non-tty
        let renderer = if !cfg!(test) && colored::control::SHOULD_COLORIZE.should_colorize() {
            Renderer::styled()
        } else {
            Renderer::plain()
        };
        let source_block = renderer.render(snippet_with_footer);
        writeln!(f, "{source_block}")
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::message::tests::{capture_emitter_output, create_messages};
    use crate::message::TextEmitter;
    use crate::settings::UnsafeFixes;

    #[test]
    fn default() {
        let mut emitter = TextEmitter::default().with_show_source(true);
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }

    #[test]
    fn fix_status() {
        let mut emitter = TextEmitter::default()
            .with_show_fix_status(true)
            .with_show_source(true);
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }

    #[test]
    fn fix_status_unsafe() {
        let mut emitter = TextEmitter::default()
            .with_show_fix_status(true)
            .with_show_source(true)
            .with_unsafe_fixes(UnsafeFixes::Enabled);
        let content = capture_emitter_output(&mut emitter, &create_messages());

        assert_snapshot!(content);
    }
}
