// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;

use colored::{Color, ColoredString, Colorize, Styles};

use ruff_text_size::{Ranged, TextRange, TextSize};
use similar::{ChangeTag, TextDiff};

use crate::diagnostics::Fix;
use ruff_source_file::{OneIndexed, SourceFile};

use crate::Diagnostic;
use crate::text_helpers::ShowNonprinting;

/// Renders a diff that shows the code fixes.
///
/// The implementation isn't fully fledged out and only used by tests. Before using in production, try
/// * Improve layout
/// * Replace tabs with spaces for a consistent experience across terminals
/// * Replace zero-width whitespaces
/// * Print a simpler diff if only a single line has changed
/// * Compute the diff from the [`crate::diagnostics::Edit`] because diff calculation is expensive.
pub(super) struct Diff<'a> {
    fix: &'a Fix,
    source_code: &'a SourceFile,
}

impl<'a> Diff<'a> {
    pub(crate) fn from_message(message: &'a Diagnostic) -> Option<Diff<'a>> {
        message.fix().map(|fix| Diff {
            source_code: message.source_file(),
            fix,
        })
    }
}

impl Display for Diff<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO(dhruvmanila): Add support for Notebook cells once it's user-facing
        let mut output = String::with_capacity(self.source_code.source_text().len());
        let mut last_end = TextSize::default();

        for edit in self.fix.edits() {
            output.push_str(
                self.source_code
                    .slice(TextRange::new(last_end, edit.start())),
            );
            output.push_str(edit.content().unwrap_or_default());
            last_end = edit.end();
        }

        output.push_str(&self.source_code.source_text()[usize::from(last_end)..]);

        let diff = TextDiff::from_lines(self.source_code.source_text(), &output);

        let grouped_ops = diff.grouped_ops(3);

        // Find the new line number with the largest number of digits to align all of the line
        // number separators.
        let last_op = grouped_ops.last().and_then(|group| group.last());
        let largest_new = last_op.map(|op| op.new_range().end).unwrap_or_default();

        let digit_with = OneIndexed::new(largest_new).unwrap_or_default().digits();

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                writeln!(f, "{:-^1$}", "-", 80)?;
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let (sign, index) = match change.tag() {
                        ChangeTag::Delete => ("-", None),
                        ChangeTag::Insert => ("+", change.new_index()),
                        ChangeTag::Equal => ("|", change.new_index()),
                    };

                    let line_style = LineStyle::from(change.tag());

                    let line = Line {
                        index: index.map(OneIndexed::from_zero_indexed),
                        width: digit_with,
                    };

                    write!(f, "{line} {sign}", sign = line_style.apply_to(sign).bold())?;

                    let mut needs_separator = true;
                    for (emphasized, value) in change.iter_strings_lossy() {
                        if needs_separator && !value.trim_end_matches(['\n', '\r']).is_empty() {
                            f.write_str(" ")?;
                            needs_separator = false;
                        }

                        let value = value.show_nonprinting();
                        let styled = line_style.apply_to(&value);
                        if emphasized {
                            write!(f, "{}", styled.underline().on_black())?;
                        } else {
                            write!(f, "{styled}",)?;
                        }
                    }
                    if change.missing_newline() {
                        writeln!(f)?;
                    }
                }
            }
        }

        Ok(())
    }
}

struct LineStyle {
    fgcolor: Option<Color>,
    style: Option<Styles>,
}

impl LineStyle {
    fn apply_to(&self, input: &str) -> ColoredString {
        let mut colored = ColoredString::from(input);
        if let Some(color) = self.fgcolor {
            colored = colored.color(color);
        }

        if let Some(style) = self.style {
            match style {
                Styles::Clear => colored.clear(),
                Styles::Bold => colored.bold(),
                Styles::Dimmed => colored.dimmed(),
                Styles::Underline => colored.underline(),
                Styles::Reversed => colored.reversed(),
                Styles::Italic => colored.italic(),
                Styles::Blink => colored.blink(),
                Styles::Hidden => colored.hidden(),
                Styles::Strikethrough => colored.strikethrough(),
            }
        } else {
            colored
        }
    }
}

impl From<ChangeTag> for LineStyle {
    fn from(value: ChangeTag) -> Self {
        match value {
            ChangeTag::Equal => LineStyle {
                fgcolor: None,
                style: Some(Styles::Dimmed),
            },
            ChangeTag::Delete => LineStyle {
                fgcolor: Some(Color::Red),
                style: None,
            },
            ChangeTag::Insert => LineStyle {
                fgcolor: Some(Color::Green),
                style: None,
            },
        }
    }
}

struct Line {
    index: Option<OneIndexed>,
    width: NonZeroUsize,
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.index {
            None => {
                for _ in 0..self.width.get() {
                    f.write_str(" ")?;
                }
                Ok(())
            }
            Some(idx) => write!(f, "{:<width$}", idx, width = self.width.get()),
        }
    }
}

/// Calculate the length of the string representation of `value`
pub(super) fn calculate_print_width(mut value: OneIndexed) -> NonZeroUsize {
    const TEN: OneIndexed = OneIndexed::from_zero_indexed(9);

    let mut width = OneIndexed::ONE;

    while value >= TEN {
        value = OneIndexed::new(value.get() / 10).unwrap_or(OneIndexed::MIN);
        width = width.checked_add(1).unwrap();
    }

    width
}
