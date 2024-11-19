use crate::registry::AsRule;
use annotate_snippets::{Level, Renderer, Snippet};
use colored::{ColoredString, Colorize};
use ruff_diagnostics::{Diagnostic, DiagnosticKind, Fix};
use ruff_source_file::{OneIndexed, SourceFile, SourceFileBuilder, SourceLocation};
use ruff_text_size::{Ranged, TextRange};
use std::cmp::Ordering;
use std::fmt;

/// Reports of each violation. They are pretty-printable and sortable.
#[derive(Debug, PartialEq, Eq)]
pub struct DiagnosticMessage {
    kind: DiagnosticKind,
    range: TextRange,
    /// The file where an error was reported.
    file: SourceFile,
    /// The rule code that was violated, expressed as a string.
    code: String,
    /// The suggested fix for the violation.
    fix: Option<Fix>,
}

impl DiagnosticMessage {
    pub fn from_ruff(file: &SourceFile, diagnostic: Diagnostic) -> Self {
        let code = diagnostic.kind.rule().noqa_code().to_string();
        Self {
            kind: diagnostic.kind,
            file: file.clone(),
            code,
            range: diagnostic.range,
            fix: diagnostic.fix,
        }
    }

    pub fn from_error<S: AsRef<str>>(filename: S, diagnostic: Diagnostic) -> Self {
        let code = diagnostic.kind.rule().noqa_code().to_string();
        Self {
            kind: diagnostic.kind,
            file: SourceFileBuilder::new(filename.as_ref(), "").finish(),
            code,
            range: diagnostic.range,
            fix: diagnostic.fix,
        }
    }
}

impl Ord for DiagnosticMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.file, self.range.start()).cmp(&(&other.file, other.range.start()))
    }
}

impl PartialOrd for DiagnosticMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ranged for DiagnosticMessage {
    fn range(&self) -> TextRange {
        self.range
    }
}

impl fmt::Display for DiagnosticMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut path: ColoredString = self.file.name().bold();
        let mut code: ColoredString = self.code.bold().bright_red();

        // Disable colours for tests, if the user requests it via env var, or non-tty
        if cfg!(test) || !colored::control::SHOULD_COLORIZE.should_colorize() {
            path = path.clear();
            code = code.clear();
        };

        let message = self.kind.body.as_str();
        let suggestion = &self.kind.suggestion;
        if self.range != TextRange::default() {
            format_violation(self, f, &self.range, message, suggestion, &path, &code)
        } else {
            write!(f, "{path}: {code} {message}")
        }
    }
}

fn format_violation(
    diagnostic: &DiagnosticMessage,
    f: &mut fmt::Formatter,
    range: &TextRange,
    message: &str,
    suggestion: &Option<String>,
    path: &ColoredString,
    code: &ColoredString,
) -> fmt::Result {
    let source_code = diagnostic.file.to_source_code();
    let content_start_index = source_code.line_index(range.start());
    let mut start_index = content_start_index.saturating_sub(2);

    // Trim leading empty lines.
    while start_index < content_start_index {
        if !source_code.line_text(start_index).trim().is_empty() {
            break;
        }
        start_index = start_index.saturating_add(1);
    }

    let content_end_index = source_code.line_index(range.end());
    let mut end_index = content_end_index
        .saturating_add(2)
        .min(OneIndexed::from_zero_indexed(source_code.line_count()));

    // Trim following empty lines.
    while end_index > content_end_index {
        if !source_code.line_text(end_index).trim().is_empty() {
            break;
        }
        end_index = end_index.saturating_sub(1);
    }

    let start_offset = source_code.line_start(start_index);
    let end_offset = source_code.line_end(end_index);

    let source = source_code.slice(TextRange::new(start_offset, end_offset));
    let message_range = range - start_offset;

    let start_char = source[TextRange::up_to(message_range.start())]
        .chars()
        .count();
    let end_char = source[TextRange::up_to(message_range.end())]
        .chars()
        .count();

    // Some annoyance here: we *have* to have some level prefix to our
    // message. Might be fixed in future version of annotate-snippets
    // -- or we use an earlier version with more control.
    // Also, we could use `.origin(path)` to get the filename and
    // line:col automatically, but there is currently a bug in the
    // library when annotating the start of the line
    let SourceLocation { row, column } = source_code.source_location(range.start());
    let message_line = format!("{path}:{row}:{column}: {code} {message}");
    let snippet = Level::Warning.title(&message_line).snippet(
        Snippet::source(source)
            .line_start(start_index.get())
            .annotation(Level::Error.span(start_char..end_char).label(code)),
    );

    let snippet_with_footer = if let Some(s) = suggestion {
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
    writeln!(f, "{source_block}")?;

    Ok(())
}
