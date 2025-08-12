pub use azure::AzureEmitter;
pub use github::GithubEmitter;
pub use gitlab::GitlabEmitter;
pub use grouped::GroupedEmitter;
pub use json::JsonEmitter;
pub use json_lines::JsonLinesEmitter;
pub use junit::JunitEmitter;
pub use pylint::PylintEmitter;
pub use rdjson::RdjsonEmitter;
pub use sarif::SarifEmitter;
pub use text::TextEmitter;

mod azure;
mod diff;
mod github;
mod gitlab;
mod grouped;
mod json;
mod json_lines;
mod junit;
mod pylint;
mod rdjson;
mod sarif;
mod text;

use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Deref;

use crate::diagnostic_message::DiagnosticMessage;
use ruff_source_file::SourceLocation;

/// Display format for a [`DiagnosticMessage`]s.
///
/// The emitter serializes a slice of [`DiagnosticMessage`]'s and writes them to a [`Write`].
pub trait Emitter {
    /// Serializes the `messages` and writes the output to `writer`.
    fn emit(
        &mut self,
        writer: &mut dyn Write,
        messages: &[DiagnosticMessage],
    ) -> anyhow::Result<()>;
}

struct MessageWithLocation<'a> {
    message: &'a DiagnosticMessage,
    start_location: SourceLocation,
}

impl Deref for MessageWithLocation<'_> {
    type Target = DiagnosticMessage;

    fn deref(&self) -> &Self::Target {
        self.message
    }
}

fn group_messages_by_filename(
    messages: &[DiagnosticMessage],
) -> BTreeMap<&str, Vec<MessageWithLocation<'_>>> {
    let mut grouped_messages = BTreeMap::default();
    for message in messages {
        grouped_messages
            .entry(message.filename())
            .or_insert_with(Vec::new)
            .push(MessageWithLocation {
                message,
                start_location: message.compute_start_location(),
            });
    }
    grouped_messages
}

#[cfg(test)]
mod tests {
    use ruff_diagnostics::{Diagnostic, DiagnosticKind, Edit, Fix};
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::{TextRange, TextSize};

    use crate::message::{DiagnosticMessage, Emitter};

    pub(super) fn create_messages() -> Vec<DiagnosticMessage> {
        let test_contents = r#"module test
implicit none

contains
  subroutine foo
    implicit none
  end subroutine
end module
"#;

        let superfluous_implicit_none = Diagnostic::new(
            DiagnosticKind {
                name: "SuperfluousImplicitNone".to_string(),
                body: "'implicit none' set on the enclosing module".to_string(),
                suggestion: Some("Remove unnecessary 'implicit none'".to_string()),
            },
            TextRange::new(TextSize::from(57), TextSize::from(70)),
        )
        .with_fix(Fix::unsafe_edit(Edit::range_deletion(TextRange::new(
            TextSize::from(57),
            TextSize::from(70),
        ))));

        let unnamed_end_statement = Diagnostic::new(
            DiagnosticKind {
                name: "UnnamedEndStatement".to_string(),
                body: "end statement should read 'end subroutine foo'".to_string(),
                suggestion: None,
            },
            TextRange::new(TextSize::from(73), TextSize::from(87)),
        );

        let test_source = SourceFileBuilder::new("test.f90", test_contents).finish();

        let file_2 = r"integer*4 foo; end";

        let star_kind = Diagnostic::new(
            DiagnosticKind {
                name: "StarKind".to_string(),
                body: "integer*4 is non-standard, use integer(4)".to_string(),
                suggestion: None,
            },
            TextRange::new(TextSize::from(7), TextSize::from(8)),
        );

        let file_2_source = SourceFileBuilder::new("star_kind.f90", file_2).finish();

        vec![
            DiagnosticMessage::from_ruff(&test_source, superfluous_implicit_none),
            DiagnosticMessage::from_ruff(&test_source, unnamed_end_statement),
            DiagnosticMessage::from_ruff(&file_2_source, star_kind),
        ]
    }

    pub(super) fn capture_emitter_output(
        emitter: &mut dyn Emitter,
        messages: &[DiagnosticMessage],
    ) -> String {
        let mut output: Vec<u8> = Vec::new();
        emitter.emit(&mut output, messages).unwrap();

        String::from_utf8(output).expect("Output to be valid UTF-8")
    }
}
