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

use crate::diagnostics::Diagnostic;
use ruff_source_file::SourceLocation;

/// Display format for a [`Diagnostic`]s.
///
/// The emitter serializes a slice of [`Diagnostic`]'s and writes them to a [`Write`].
pub trait Emitter {
    /// Serializes the `messages` and writes the output to `writer`.
    fn emit(&mut self, writer: &mut dyn Write, messages: &[Diagnostic]) -> anyhow::Result<()>;
}

struct MessageWithLocation<'a> {
    message: &'a Diagnostic,
    start_location: SourceLocation,
}

impl Deref for MessageWithLocation<'_> {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        self.message
    }
}

fn group_messages_by_filename(
    messages: &[Diagnostic],
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
        diagnostics::{Edit, Fix, test_diagnostic_builder},
        rules::Rule,
    };
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::{TextRange, TextSize};

    use super::Emitter;
    use crate::Diagnostic;

    pub(super) fn create_messages() -> Vec<Diagnostic> {
        let test_contents = r#"module test
implicit none

contains
  subroutine foo
    implicit none
  end subroutine
end module
"#;

        let test_source = SourceFileBuilder::new("test.f90", test_contents).finish();

        let superfluous_implicit_none = test_diagnostic_builder(
            Rule::SuperfluousImplicitNone,
            "'implicit none' set on the enclosing module",
            TextRange::new(TextSize::from(57), TextSize::from(70)),
        )
        .with_suggestion(Some("Remove unnecessary 'implicit none'".to_string()))
        .with_fix(Fix::unsafe_edit(Edit::range_deletion(TextRange::new(
            TextSize::from(57),
            TextSize::from(70),
        ))))
        .with_file(test_source.clone());

        let unnamed_end_statement = test_diagnostic_builder(
            Rule::UnnamedEndStatement,
            "end statement should read 'end subroutine foo'",
            TextRange::new(TextSize::from(73), TextSize::from(87)),
        )
        .with_file(test_source);

        let file_2 = r"integer*4 foo; end";
        let file_2_source = SourceFileBuilder::new("star_kind.f90", file_2).finish();

        let star_kind = test_diagnostic_builder(
            Rule::StarKind,
            "integer*4 is non-standard, use integer(4)",
            TextRange::new(TextSize::from(7), TextSize::from(8)),
        )
        .with_file(file_2_source);

        vec![superfluous_implicit_none, unnamed_end_statement, star_kind]
    }

    pub(super) fn capture_emitter_output(
        emitter: &mut dyn Emitter,
        messages: &[Diagnostic],
    ) -> String {
        let mut output: Vec<u8> = Vec::new();
        emitter.emit(&mut output, messages).unwrap();

        String::from_utf8(output).expect("Output to be valid UTF-8")
    }
}
