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

use super::diagnostic_message::DiagnosticMessage;
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
    use crate::{
        diagnostics::{Edit, Fix},
        rules::Rule,
    };
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::{TextRange, TextSize};

    use super::{DiagnosticMessage, Emitter};

    pub(super) fn create_messages() -> Vec<DiagnosticMessage> {
        let test_contents = r#"module test
implicit none

contains
  subroutine foo
    implicit none
  end subroutine
end module
"#;

        let test_source = SourceFileBuilder::new("test.f90", test_contents).finish();

        let superfluous_implicit_none = DiagnosticMessage {
            rule: Rule::SuperfluousImplicitNone,
            body: "'implicit none' set on the enclosing module".to_string(),
            suggestion: Some("Remove unnecessary 'implicit none'".to_string()),
            range: TextRange::new(TextSize::from(57), TextSize::from(70)),
            file: test_source.clone(),
            code: Rule::SuperfluousImplicitNone.noqa_code().to_string(),
            fix: Some(Fix::unsafe_edit(Edit::range_deletion(TextRange::new(
                TextSize::from(57),
                TextSize::from(70),
            )))),
        };

        let unnamed_end_statement = DiagnosticMessage {
            rule: Rule::UnnamedEndStatement,
            body: "end statement should read 'end subroutine foo'".to_string(),
            suggestion: None,
            range: TextRange::new(TextSize::from(73), TextSize::from(87)),
            file: test_source,
            code: Rule::UnnamedEndStatement.noqa_code().to_string(),
            fix: None,
        };

        let file_2 = r"integer*4 foo; end";
        let file_2_source = SourceFileBuilder::new("star_kind.f90", file_2).finish();

        let star_kind = DiagnosticMessage {
            rule: Rule::StarKind,
            body: "integer*4 is non-standard, use integer(4)".to_string(),
            suggestion: None,
            range: TextRange::new(TextSize::from(7), TextSize::from(8)),
            file: file_2_source,
            code: Rule::StarKind.noqa_code().to_string(),
            fix: None,
        };

        vec![superfluous_implicit_none, unnamed_end_statement, star_kind]
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
