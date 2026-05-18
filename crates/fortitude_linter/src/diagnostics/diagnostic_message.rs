use crate::diagnostics::{Diagnostic, Fix};
use crate::rules::Rule;
use ruff_source_file::{SourceFile, SourceFileBuilder, SourceLocation};
use ruff_text_size::{Ranged, TextRange};
use std::cmp::Ordering;

/// Reports of each violation. They are pretty-printable and sortable.
#[derive(Debug, PartialEq, Eq)]
pub struct DiagnosticMessage {
    /// The file where an error was reported.
    pub file: SourceFile,
    pub diagnostic: Diagnostic,
}

impl DiagnosticMessage {
    pub fn from_ruff(file: &SourceFile, diagnostic: Diagnostic) -> Self {
        Self {
            file: file.clone(),
            diagnostic,
        }
    }

    pub fn from_error<S: AsRef<str>>(filename: S, diagnostic: Diagnostic) -> Self {
        Self {
            file: SourceFileBuilder::new(filename.as_ref(), "").finish(),
            diagnostic,
        }
    }

    /// Returns the name used to represent the diagnostic.
    pub fn name(&self) -> &str {
        self.diagnostic.rule().into()
    }

    /// Returns the message body to display to the user.
    pub fn body(&self) -> &str {
        self.diagnostic.body()
    }

    /// Returns the rule code that was violated.
    pub fn code(&self) -> &str {
        self.diagnostic.code()
    }

    /// Returns the fix suggestion for the violation.
    pub fn suggestion(&self) -> Option<&str> {
        self.diagnostic.suggestion()
    }

    /// Returns the [`Fix`] for the message, if there is any.
    pub fn fix(&self) -> Option<&Fix> {
        self.diagnostic.fix()
    }

    /// Returns `true` if the message contains a [`Fix`].
    pub fn fixable(&self) -> bool {
        self.fix().is_some()
    }

    /// Returns the [`Rule`] corresponding to the diagnostic message.
    pub fn rule(&self) -> Rule {
        self.diagnostic.rule()
    }

    /// Returns the filename for the message.
    pub fn filename(&self) -> &str {
        self.source_file().name()
    }

    /// Computes the start source location for the message.
    pub fn compute_start_location(&self) -> SourceLocation {
        self.source_file()
            .to_source_code()
            .source_location(self.start())
    }

    /// Computes the end source location for the message.
    #[allow(dead_code)]
    pub fn compute_end_location(&self) -> SourceLocation {
        self.source_file()
            .to_source_code()
            .source_location(self.end())
    }

    /// Returns the [`SourceFile`] which the message belongs to.
    pub fn source_file(&self) -> &SourceFile {
        &self.file
    }

    /// Returns the URL for the rule documentation
    pub fn to_fortitude_url(&self) -> String {
        self.diagnostic.to_fortitude_url()
    }
}

impl Ord for DiagnosticMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.file, self.range().start()).cmp(&(&other.file, other.range().start()))
    }
}

impl PartialOrd for DiagnosticMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ranged for DiagnosticMessage {
    fn range(&self) -> TextRange {
        self.diagnostic.range()
    }
}
