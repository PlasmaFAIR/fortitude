use crate::diagnostics::{Diagnostic, Fix};
use crate::registry::AsRule;
use crate::rules::Rule;
use ruff_source_file::{SourceFile, SourceFileBuilder, SourceLocation};
use ruff_text_size::{Ranged, TextRange};
use std::cmp::Ordering;

/// Reports of each violation. They are pretty-printable and sortable.
#[derive(Debug, PartialEq, Eq)]
pub struct DiagnosticMessage {
    pub body: String,
    pub suggestion: Option<String>,
    pub range: TextRange,
    /// The file where an error was reported.
    pub file: SourceFile,
    /// The rule code that was violated, expressed as a string.
    pub code: String,
    /// The suggested fix for the violation.
    pub fix: Option<Fix>,
    pub rule: Rule,
}

impl DiagnosticMessage {
    pub fn from_ruff(file: &SourceFile, diagnostic: Diagnostic) -> Self {
        let rule = diagnostic.rule();
        let code = diagnostic.rule().noqa_code().to_string();
        Self {
            body: diagnostic.body,
            suggestion: diagnostic.suggestion,
            file: file.clone(),
            code,
            range: diagnostic.range,
            fix: diagnostic.fix,
            rule,
        }
    }

    pub fn from_error<S: AsRef<str>>(filename: S, diagnostic: Diagnostic) -> Self {
        let rule = diagnostic.rule();
        let code = diagnostic.rule().noqa_code().to_string();
        Self {
            body: diagnostic.body,
            suggestion: diagnostic.suggestion,
            file: SourceFileBuilder::new(filename.as_ref(), "").finish(),
            code,
            range: diagnostic.range,
            fix: diagnostic.fix,
            rule,
        }
    }

    /// Returns the name used to represent the diagnostic.
    pub fn name(&self) -> &str {
        self.rule.into()
    }

    /// Returns the message body to display to the user.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the rule code that was violated.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the fix suggestion for the violation.
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Returns the [`Fix`] for the message, if there is any.
    pub fn fix(&self) -> Option<&Fix> {
        self.fix.as_ref()
    }

    /// Returns `true` if the message contains a [`Fix`].
    pub fn fixable(&self) -> bool {
        self.fix().is_some()
    }

    /// Returns the [`Rule`] corresponding to the diagnostic message.
    pub fn rule(&self) -> Rule {
        self.rule
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
        format!(
            "{}/en/stable/rules/{}",
            env!("CARGO_PKG_HOMEPAGE"),
            self.rule()
        )
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
