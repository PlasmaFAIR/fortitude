// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

mod message;
mod stylesheet;
pub mod violation;

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    ops::{Add, AddAssign},
    sync::Arc,
};

use ruff_source_file::{LineColumn, SourceFile};
use rustc_hash::FxHashMap;

use anyhow::Result;
use ruff_annotate_snippets::Level as AnnotateLevel;
use ruff_text_size::{Ranged, TextRange};
use serde::Serialize;

use crate::{fix::FixTable, rules::Rule, settings::OutputFormat, traits::TextRanged};

pub use message::{DisplayDiagnostic, DisplayDiagnostics, render_diagnostics};
pub use violation::{AlwaysFixableViolation, FixAvailability, Violation, ViolationMetadata};

// Re-export some things from ruff
pub use ruff_diagnostics::{Applicability, Edit, Fix, IsolationLevel, SourceMap, SourceMarker};

#[derive(Debug, Default, PartialEq)]
pub struct Diagnostics {
    pub messages: Vec<Diagnostic>,
    pub fixed: FixMap,
}

impl Diagnostics {
    pub fn new(messages: Vec<Diagnostic>) -> Self {
        Self {
            messages,
            fixed: FixMap::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.fixed.is_empty()
    }
}

impl Add for Diagnostics {
    type Output = Diagnostics;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl AddAssign for Diagnostics {
    fn add_assign(&mut self, other: Self) {
        self.messages.extend(other.messages);
        self.fixed += other.fixed;
    }
}

/// A collection of fixes indexed by file path.
#[derive(Debug, Default, PartialEq)]
pub struct FixMap(FxHashMap<String, FixTable>);

impl FixMap {
    /// Returns `true` if there are no fixes in the map.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the fixes in the map, along with the file path.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FixTable)> {
        self.0.iter()
    }

    /// Returns an iterator over the fixes in the map.
    pub fn values(&self) -> impl Iterator<Item = &FixTable> {
        self.0.values()
    }
}

impl FromIterator<(String, FixTable)> for FixMap {
    fn from_iter<T: IntoIterator<Item = (String, FixTable)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .filter(|(_, fixes)| !fixes.is_empty())
                .collect(),
        )
    }
}

impl AddAssign for FixMap {
    fn add_assign(&mut self, rhs: Self) {
        for (filename, fixed) in rhs.0 {
            if fixed.is_empty() {
                continue;
            }
            let fixed_in_file = self.0.entry(filename).or_default();
            for (rule, count) in fixed {
                if count > 0 {
                    *fixed_in_file.entry(rule).or_default() += count;
                }
            }
        }
    }
}

/// A collection of information that can be rendered into a diagnostic.
///
/// A diagnostic is a collection of information gathered by a tool intended
/// for presentation to an end user, and which describes a group of related
/// characteristics in the inputs given to the tool. Typically, but not always,
/// a characteristic is a deficiency. An example of a characteristic that is
/// _not_ a deficiency is the `reveal_type` diagnostic for our type checker.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Diagnostic {
    /// The actual diagnostic.
    ///
    /// We box the diagnostic since it is somewhat big.
    inner: Arc<DiagnosticInner>,
}

impl Diagnostic {
    /// Create a new diagnostic with the given identifier, severity and
    /// message.
    ///
    /// The identifier should be something that uniquely identifies the _type_
    /// of diagnostic being reported. It should be usable as a reference point
    /// for humans communicating about diagnostic categories. It will also
    /// appear in the output when this diagnostic is rendered.
    ///
    /// The severity should describe the assumed level of importance to an end
    /// user.
    ///
    /// The message is meant to be read by end users. The primary message
    /// is meant to be a single terse description (usually a short phrase)
    /// describing the group of related characteristics that the diagnostic
    /// describes. Stated differently, if only one thing from a diagnostic can
    /// be shown to an end user in a particular context, it is the primary
    /// message.
    ///
    /// # Types implementing `IntoDiagnosticMessage`
    ///
    /// Callers can pass anything that implements `std::fmt::Display`
    /// directly. If callers want or need to avoid cloning the diagnostic
    /// message, then they can also pass a `DiagnosticMessage` directly.
    pub fn new<'a>(
        id: DiagnosticId,
        severity: Severity,
        message: impl IntoDiagnosticMessage + 'a,
    ) -> Diagnostic {
        let inner = Arc::new(DiagnosticInner {
            id,
            severity,
            message: message.into_diagnostic_message(),
            custom_concise_message: None,
            documentation_url: None,
            annotations: vec![],
            subs: vec![],
            fix: None,
            secondary_code: None,
            header_offset: 0,
        });
        Diagnostic { inner }
    }

    /// Creates a `Diagnostic` for a syntax error.
    ///
    /// Unlike the more general [`Diagnostic::new`], this requires a [`Span`] and a [`TextRange`]
    /// attached to it.
    pub fn invalid_syntax(
        span: impl Into<Span>,
        message: impl IntoDiagnosticMessage,
        range: impl Ranged,
    ) -> Diagnostic {
        let mut diag = Diagnostic::new(DiagnosticId::InvalidSyntax, Severity::Error, message);
        let span = span.into().with_range(range.range());
        diag.annotate(Annotation::primary(span));
        diag
    }

    /// Add an annotation to this diagnostic.
    ///
    /// Annotations for a diagnostic are optional, but if any are added,
    /// callers should strive to make at least one of them primary. That is, it
    /// should be constructed via [`Annotation::primary`]. A diagnostic with no
    /// primary annotations is allowed, but its rendering may be sub-optimal.
    pub fn annotate(&mut self, ann: Annotation) {
        Arc::make_mut(&mut self.inner).annotations.push(ann);
    }

    /// Adds an "info" sub-diagnostic with the given message.
    ///
    /// If callers want to add an "info" sub-diagnostic with annotations, then
    /// create a [`SubDiagnostic`] manually and use [`Diagnostic::sub`] to
    /// attach it to a parent diagnostic.
    ///
    /// An "info" diagnostic is useful when contextualizing or otherwise
    /// helpful information can be added to help end users understand the
    /// main diagnostic message better. For example, if a the main diagnostic
    /// message is about a function call being invalid, a useful "info"
    /// sub-diagnostic could show the function definition (or only the relevant
    /// parts of it).
    ///
    /// # Types implementing `IntoDiagnosticMessage`
    ///
    /// Callers can pass anything that implements `std::fmt::Display`
    /// directly. If callers want or need to avoid cloning the diagnostic
    /// message, then they can also pass a `DiagnosticMessage` directly.
    pub fn info<'a>(&mut self, message: impl IntoDiagnosticMessage + 'a) {
        self.sub(SubDiagnostic::new(SubDiagnosticSeverity::Info, message));
    }

    /// Adds a "help" sub-diagnostic with the given message.
    ///
    /// See the closely related [`Diagnostic::info`] method for more details.
    pub fn help<'a>(&mut self, message: impl IntoDiagnosticMessage + 'a) {
        self.sub(SubDiagnostic::new(SubDiagnosticSeverity::Help, message));
    }

    /// Adds a "sub" diagnostic to this diagnostic.
    ///
    /// This is useful when a sub diagnostic has its own annotations attached
    /// to it. For the simpler case of a sub-diagnostic with only a message,
    /// using a method like [`Diagnostic::info`] may be more convenient.
    pub fn sub(&mut self, sub: SubDiagnostic) {
        Arc::make_mut(&mut self.inner).subs.push(sub);
    }

    /// Return a `std::fmt::Display` implementation that renders this
    /// diagnostic into a human readable format.
    ///
    /// Note that this `Display` impl includes a trailing line terminator, so
    /// callers should prefer using this with `write!` instead of `writeln!`.
    pub fn display<'a>(&'a self, config: &'a DisplayDiagnosticConfig) -> DisplayDiagnostic<'a> {
        DisplayDiagnostic::new(config, self)
    }

    /// Returns the identifier for this diagnostic.
    pub fn id(&self) -> DiagnosticId {
        self.inner.id
    }

    /// Returns the associated rule, if any
    pub fn rule(&self) -> Option<Rule> {
        match self.inner.id {
            DiagnosticId::Lint(rule) => Some(rule),
            _ => None,
        }
    }

    /// Returns the primary message for this diagnostic.
    ///
    /// A diagnostic always has a message, but it may be empty.
    pub fn primary_message(&self) -> &str {
        self.inner.message.as_str()
    }

    /// Introspects this diagnostic and returns what kind of "primary" message
    /// it contains for concise formatting.
    ///
    /// When we concisely format diagnostics, we likely want to not only
    /// include the primary diagnostic message but also the message attached
    /// to the primary annotation. In particular, the primary annotation often
    /// contains *essential* information or context for understanding the
    /// diagnostic.
    ///
    /// The type returned implements the `std::fmt::Display` trait. In most
    /// cases, just converting it to a string (or printing it) will do what
    /// you want.
    pub fn concise_message(&self) -> ConciseMessage<'_> {
        if let Some(custom_message) = &self.inner.custom_concise_message {
            return ConciseMessage::Custom(custom_message.as_str());
        }

        let main = self.inner.message.as_str();
        let annotation = self
            .primary_annotation()
            .and_then(|ann| ann.get_message())
            .unwrap_or_default();
        if annotation.is_empty() {
            ConciseMessage::MainDiagnostic(main)
        } else {
            ConciseMessage::Both { main, annotation }
        }
    }

    /// Set a custom message for the concise formatting of this diagnostic.
    ///
    /// This overrides the default behavior of generating a concise message
    /// from the main diagnostic message and the primary annotation.
    pub fn set_concise_message(&mut self, message: impl IntoDiagnosticMessage) {
        Arc::make_mut(&mut self.inner).custom_concise_message =
            Some(message.into_diagnostic_message());
    }

    /// Returns the severity of this diagnostic.
    ///
    /// Note that this may be different than the severity of sub-diagnostics.
    pub fn severity(&self) -> Severity {
        self.inner.severity
    }

    /// Returns a shared borrow of the "primary" annotation of this diagnostic
    /// if one exists.
    ///
    /// When there are multiple primary annotations, then the first one that
    /// was added to this diagnostic is returned.
    pub fn primary_annotation(&self) -> Option<&Annotation> {
        self.inner.annotations.iter().find(|ann| ann.is_primary)
    }

    /// Returns a mutable borrow of the "primary" annotation of this diagnostic
    /// if one exists.
    ///
    /// When there are multiple primary annotations, then the first one that
    /// was added to this diagnostic is returned.
    pub fn primary_annotation_mut(&mut self) -> Option<&mut Annotation> {
        Arc::make_mut(&mut self.inner)
            .annotations
            .iter_mut()
            .find(|ann| ann.is_primary)
    }

    /// Returns a mutable borrow of all annotations of this diagnostic.
    pub fn annotations_mut(&mut self) -> impl Iterator<Item = &mut Annotation> {
        Arc::make_mut(&mut self.inner).annotations.iter_mut()
    }

    /// Returns the "primary" span of this diagnostic if one exists.
    ///
    /// When there are multiple primary spans, then the first one that was
    /// added to this diagnostic is returned.
    pub fn primary_span(&self) -> Option<Span> {
        self.primary_annotation().map(|ann| ann.span.clone())
    }

    /// Returns a reference to the primary span of this diagnostic.
    pub fn primary_span_ref(&self) -> Option<&Span> {
        self.primary_annotation().map(|ann| &ann.span)
    }

    /// Returns the tags from the primary annotation of this diagnostic if it exists.
    pub fn primary_tags(&self) -> Option<&[DiagnosticTag]> {
        self.primary_annotation().map(|ann| ann.tags.as_slice())
    }

    /// Returns the "primary" span of this diagnostic, panicking if it does not exist.
    ///
    /// See [`Diagnostic::primary_span`] for more details.
    pub fn expect_primary_span(&self) -> Span {
        self.primary_span().expect("Expected a primary span")
    }

    /// Returns a key that can be used to sort two diagnostics into the canonical order
    /// in which they should appear when rendered.
    pub fn rendering_sort_key<'a>(&'a self) -> impl Ord + 'a {
        RenderingSortKey { diagnostic: self }
    }

    /// Returns all annotations, skipping the first primary annotation.
    pub fn secondary_annotations(&self) -> impl Iterator<Item = &Annotation> {
        secondary_annotations(self.inner.annotations.iter())
    }

    pub fn sub_diagnostics(&self) -> &[SubDiagnostic] {
        &self.inner.subs
    }

    /// Returns a mutable borrow of the sub-diagnostics of this diagnostic.
    pub fn sub_diagnostics_mut(&mut self) -> impl Iterator<Item = &mut SubDiagnostic> {
        Arc::make_mut(&mut self.inner).subs.iter_mut()
    }

    /// Returns the fix for this diagnostic if it exists.
    pub fn fix(&self) -> Option<&Fix> {
        self.inner.fix.as_ref()
    }

    /// Set the fix for this diagnostic.
    pub fn set_fix(&mut self, fix: Fix) {
        debug_assert!(
            self.primary_span().is_some(),
            "Expected a source file for a diagnostic with a fix"
        );
        Arc::make_mut(&mut self.inner).fix = Some(fix);
    }

    /// If `fix` is `Some`, set the fix for this diagnostic.
    pub fn set_optional_fix(&mut self, fix: Option<Fix>) {
        if let Some(fix) = fix {
            self.set_fix(fix);
        }
    }

    #[must_use]
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.set_fix(fix);
        self
    }

    /// Remove the fix for this diagnostic.
    pub fn remove_fix(&mut self) {
        Arc::make_mut(&mut self.inner).fix = None;
    }

    /// Returns `true` if the diagnostic contains a [`Fix`].
    pub fn fixable(&self) -> bool {
        self.fix().is_some()
    }

    /// Returns `true` if the diagnostic is [`fixable`](Diagnostic::fixable) and applies at the
    /// configured applicability level.
    pub fn has_applicable_fix(&self, fix_applicability: Applicability) -> bool {
        self.fix().is_some_and(|fix| fix.applies(fix_applicability))
    }

    pub fn documentation_url(&self) -> Option<&str> {
        self.inner.documentation_url.as_deref()
    }

    pub fn set_documentation_url(&mut self, url: Option<String>) {
        Arc::make_mut(&mut self.inner).documentation_url = url;
    }

    /// Returns the secondary code for the diagnostic if it exists.
    ///
    /// The "primary" code for the diagnostic is its lint name, the secondary
    /// code is the short code.
    pub fn secondary_code(&self) -> Option<&SecondaryCode> {
        self.inner.secondary_code.as_ref()
    }

    /// Returns the secondary code for the diagnostic if it exists, or the lint name otherwise.
    ///
    /// This is a common pattern for Ruff diagnostics, which want to use the noqa code in general,
    /// but fall back on the `invalid-syntax` identifier for syntax errors, which don't have
    /// secondary codes.
    pub fn secondary_code_or_id(&self) -> &str {
        self.secondary_code()
            .map_or_else(|| self.inner.id.as_str(), SecondaryCode::as_str)
    }

    /// Set the secondary code for this diagnostic.
    pub fn set_secondary_code(&mut self, code: SecondaryCode) {
        Arc::make_mut(&mut self.inner).secondary_code = Some(code);
    }

    /// Returns the name used to represent the diagnostic.
    pub fn name(&self) -> &'static str {
        self.id().as_str()
    }

    /// Returns `true` if `self` is a syntax error message.
    pub fn is_invalid_syntax(&self) -> bool {
        self.id().is_invalid_syntax()
    }

    /// Returns the message of the first sub-diagnostic with a `Help` severity.
    ///
    /// Note that this is used as the fix title/suggestion for some of Ruff's output formats, but in
    /// general this is not the guaranteed meaning of such a message.
    pub fn first_help_text(&self) -> Option<&str> {
        self.sub_diagnostics()
            .iter()
            .find(|sub| matches!(sub.inner.severity, SubDiagnosticSeverity::Help))
            .map(|sub| sub.inner.message.as_str())
    }

    /// Returns the filename for the message.
    ///
    /// Panics if the diagnostic has no primary span.
    pub fn expect_filename(&self) -> String {
        self.expect_primary_span().file().name().to_string()
    }

    /// Computes the start source location for the message.
    ///
    /// Returns None if the diagnostic has no primary span, or if the span has
    /// no range.
    pub fn start_location(&self) -> Option<LineColumn> {
        Some(
            self.source_file()?
                .to_source_code()
                .line_column(self.range()?.start()),
        )
    }

    /// Computes the end source location for the message.
    ///
    /// Returns None if the diagnostic has no primary span, or if the span has
    /// no range.
    pub fn end_location(&self) -> Option<LineColumn> {
        Some(
            self.source_file()?
                .to_source_code()
                .line_column(self.range()?.end()),
        )
    }

    /// Returns the [`SourceFile`] which the message belongs to.
    pub fn source_file(&self) -> Option<&SourceFile> {
        Some(self.primary_span_ref()?.file())
    }

    /// Returns the [`SourceFile`] which the message belongs to.
    ///
    /// Panics if the diagnostic has no primary span.
    pub fn expect_source_file(&self) -> &SourceFile {
        self.source_file().expect("Expected a ruff source file")
    }

    /// Returns the [`TextRange`] for the diagnostic.
    pub fn range(&self) -> Option<TextRange> {
        self.primary_span()?.range()
    }

    /// Returns the ordering of diagnostics based on the start of their ranges, if they have any.
    ///
    /// Panics if either diagnostic has no primary span.
    pub fn start_ordering(&self, other: &Self) -> std::cmp::Ordering {
        let a = (
            self.severity().is_fatal(),
            self.expect_source_file(),
            self.range().map(|r| r.start()),
        );
        let b = (
            other.severity().is_fatal(),
            other.expect_source_file(),
            other.range().map(|r| r.start()),
        );

        a.cmp(&b)
    }

    /// Add an offset for aligning the header sigil with the line number separators in a diff.
    pub fn set_header_offset(&mut self, offset: usize) {
        Arc::make_mut(&mut self.inner).header_offset = offset;
    }
}

impl Ord for Diagnostic {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start_ordering(other)
    }
}

impl PartialOrd for Diagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(&self, &other))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct DiagnosticInner {
    id: DiagnosticId,
    documentation_url: Option<String>,
    severity: Severity,
    message: DiagnosticMessage,
    custom_concise_message: Option<DiagnosticMessage>,
    annotations: Vec<Annotation>,
    subs: Vec<SubDiagnostic>,
    fix: Option<Fix>,
    secondary_code: Option<SecondaryCode>,
    header_offset: usize,
}

struct RenderingSortKey<'a> {
    diagnostic: &'a Diagnostic,
}

impl Ord for RenderingSortKey<'_> {
    // We sort diagnostics in a way that keeps them in source order
    // and grouped by file. After that, we fall back to severity
    // (with fatal messages sorting before info messages) and then
    // finally the diagnostic ID.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let (Some(span1), Some(span2)) = (
            self.diagnostic.primary_span(),
            other.diagnostic.primary_span(),
        ) {
            let order = span1.file().cmp(span2.file());
            if order.is_ne() {
                return order;
            }

            if let (Some(range1), Some(range2)) = (span1.range(), span2.range()) {
                let order = range1.start().cmp(&range2.start());
                if order.is_ne() {
                    return order;
                }
            }
        }
        // Reverse so that, e.g., Fatal sorts before Info.
        let order = self
            .diagnostic
            .severity()
            .cmp(&other.diagnostic.severity())
            .reverse();
        if order.is_ne() {
            return order;
        }
        self.diagnostic.id().cmp(&other.diagnostic.id())
    }
}

impl PartialOrd for RenderingSortKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RenderingSortKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for RenderingSortKey<'_> {}

/// A collection of information subservient to a diagnostic.
///
/// A sub-diagnostic is always rendered after the parent diagnostic it is
/// attached to. A parent diagnostic may have many sub-diagnostics, and it is
/// guaranteed that they will not interleave with one another in rendering.
///
/// Currently, the order in which sub-diagnostics are rendered relative to one
/// another (for a single parent diagnostic) is the order in which they were
/// attached to the diagnostic.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubDiagnostic {
    /// Like with `Diagnostic`, we box the `SubDiagnostic` to make it
    /// pointer-sized.
    inner: Box<SubDiagnosticInner>,
}

impl SubDiagnostic {
    /// Create a new sub-diagnostic with the given severity and message.
    ///
    /// The severity should describe the assumed level of importance to an end
    /// user.
    ///
    /// The message is meant to be read by end users. The primary message
    /// is meant to be a single terse description (usually a short phrase)
    /// describing the group of related characteristics that the sub-diagnostic
    /// describes. Stated differently, if only one thing from a diagnostic can
    /// be shown to an end user in a particular context, it is the primary
    /// message.
    ///
    /// # Types implementing `IntoDiagnosticMessage`
    ///
    /// Callers can pass anything that implements `std::fmt::Display`
    /// directly. If callers want or need to avoid cloning the diagnostic
    /// message, then they can also pass a `DiagnosticMessage` directly.
    pub fn new<'a>(
        severity: SubDiagnosticSeverity,
        message: impl IntoDiagnosticMessage + 'a,
    ) -> SubDiagnostic {
        let inner = Box::new(SubDiagnosticInner {
            severity,
            message: message.into_diagnostic_message(),
            annotations: vec![],
        });
        SubDiagnostic { inner }
    }

    /// Add an annotation to this sub-diagnostic.
    ///
    /// Annotations for a sub-diagnostic, like for a diagnostic, are optional.
    /// If any are added, callers should strive to make at least one of them
    /// primary. That is, it should be constructed via [`Annotation::primary`].
    /// A diagnostic with no primary annotations is allowed, but its rendering
    /// may be sub-optimal.
    ///
    /// Note that it is expected to be somewhat more common for sub-diagnostics
    /// to have no annotations (e.g., a simple note) than for a diagnostic to
    /// have no annotations.
    pub fn annotate(&mut self, ann: Annotation) {
        self.inner.annotations.push(ann);
    }

    pub fn annotations(&self) -> &[Annotation] {
        &self.inner.annotations
    }

    /// Returns all annotations, skipping the first primary annotation.
    pub fn secondary_annotations(&self) -> impl Iterator<Item = &Annotation> {
        secondary_annotations(self.inner.annotations.iter())
    }

    /// Returns a mutable borrow of the annotations of this sub-diagnostic.
    pub fn annotations_mut(&mut self) -> impl Iterator<Item = &mut Annotation> {
        self.inner.annotations.iter_mut()
    }

    /// Returns a shared borrow of the "primary" annotation of this diagnostic
    /// if one exists.
    ///
    /// When there are multiple primary annotations, then the first one that
    /// was added to this diagnostic is returned.
    pub fn primary_annotation(&self) -> Option<&Annotation> {
        self.inner.annotations.iter().find(|ann| ann.is_primary)
    }

    /// Introspects this diagnostic and returns what kind of "primary" message
    /// it contains for concise formatting.
    ///
    /// When we concisely format diagnostics, we likely want to not only
    /// include the primary diagnostic message but also the message attached
    /// to the primary annotation. In particular, the primary annotation often
    /// contains *essential* information or context for understanding the
    /// diagnostic.
    ///
    /// The type returned implements the `std::fmt::Display` trait. In most
    /// cases, just converting it to a string (or printing it) will do what
    /// you want.
    pub fn concise_message(&self) -> ConciseMessage<'_> {
        let main = self.inner.message.as_str();
        let annotation = self
            .primary_annotation()
            .and_then(|ann| ann.get_message())
            .unwrap_or_default();
        if annotation.is_empty() {
            ConciseMessage::MainDiagnostic(main)
        } else {
            ConciseMessage::Both { main, annotation }
        }
    }

    pub fn severity(&self) -> SubDiagnosticSeverity {
        self.inner.severity
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct SubDiagnosticInner {
    severity: SubDiagnosticSeverity,
    message: DiagnosticMessage,
    annotations: Vec<Annotation>,
}

/// Returns all annotations, skipping the first primary annotation.
fn secondary_annotations<'a>(
    annotations: impl Iterator<Item = &'a Annotation>,
) -> impl Iterator<Item = &'a Annotation> {
    let mut seen_primary = false;
    annotations.filter(move |ann| {
        if seen_primary {
            true
        } else if ann.is_primary {
            seen_primary = true;
            false
        } else {
            true
        }
    })
}

/// A pointer to a subsequence in the end user's input.
///
/// Also known as an annotation, the pointer can optionally contain a short
/// message, typically describing in general terms what is being pointed to.
///
/// An annotation is either primary or secondary, depending on whether it was
/// constructed via [`Annotation::primary`] or [`Annotation::secondary`].
/// Semantically, a primary annotation is meant to point to the "locus" of a
/// diagnostic. Visually, the difference between a primary and a secondary
/// annotation is usually just a different form of highlighting on the
/// corresponding span.
///
/// # Advice
///
/// The span on an annotation should be as _specific_ as possible. For example,
/// if there is a problem with a function call because one of its arguments has
/// an invalid type, then the span should point to the specific argument and
/// not to the entire function call.
///
/// Messages attached to annotations should also be as brief and specific as
/// possible. Long messages could negative impact the quality of rendering.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Annotation {
    /// The span of this annotation, corresponding to some subsequence of the
    /// user's input that we want to highlight.
    span: Span,
    /// An optional message associated with this annotation's span.
    ///
    /// When present, rendering will include this message in the output and
    /// draw a line between the highlighted span and the message.
    message: Option<DiagnosticMessage>,
    /// Whether this annotation is "primary" or not. When it isn't primary, an
    /// annotation is said to be "secondary."
    is_primary: bool,
    /// The diagnostic tags associated with this annotation.
    tags: Vec<DiagnosticTag>,
    /// Whether the snippet for this annotation should be hidden.
    ///
    /// When set, rendering will only include the file's name and (optional) range. Everything else
    /// is omitted, including any file snippet or message.
    hide_snippet: bool,
}

impl Annotation {
    /// Create a "primary" annotation.
    ///
    /// A primary annotation is meant to highlight the "locus" of a diagnostic.
    /// That is, it should point to something in the end user's input that is
    /// the subject or "point" of a diagnostic.
    ///
    /// A diagnostic may have many primary annotations. A diagnostic may not
    /// have any annotations, but if it does, at least one _ought_ to be
    /// primary.
    pub fn primary(span: Span) -> Annotation {
        Annotation {
            span,
            message: None,
            is_primary: true,
            tags: Vec::new(),
            hide_snippet: false,
        }
    }

    /// Create a "secondary" annotation.
    ///
    /// A secondary annotation is meant to highlight relevant context for a
    /// diagnostic, but not to point to the "locus" of the diagnostic.
    ///
    /// A diagnostic with only secondary annotations is usually not sensible,
    /// but it is allowed and will produce a reasonable rendering.
    pub fn secondary(span: Span) -> Annotation {
        Annotation {
            span,
            message: None,
            is_primary: false,
            tags: Vec::new(),
            hide_snippet: false,
        }
    }

    /// Attach a message to this annotation.
    ///
    /// An annotation without a message will still have a presence in
    /// rendering. In particular, it will highlight the span association with
    /// this annotation in some way.
    ///
    /// When a message is attached to an annotation, then it will be associated
    /// with the highlighted span in some way during rendering.
    ///
    /// # Types implementing `IntoDiagnosticMessage`
    ///
    /// Callers can pass anything that implements `std::fmt::Display`
    /// directly. If callers want or need to avoid cloning the diagnostic
    /// message, then they can also pass a `DiagnosticMessage` directly.
    pub fn message<'a>(self, message: impl IntoDiagnosticMessage + 'a) -> Annotation {
        let message = Some(message.into_diagnostic_message());
        Annotation { message, ..self }
    }

    /// Sets the message on this annotation.
    ///
    /// If one was already set, then this overwrites it.
    ///
    /// This is useful if one needs to set the message on an annotation,
    /// and all one has is a `&mut Annotation`. For example, via
    /// `Diagnostic::primary_annotation_mut`.
    pub fn set_message<'a>(&mut self, message: impl IntoDiagnosticMessage + 'a) {
        self.message = Some(message.into_diagnostic_message());
    }

    /// Returns the message attached to this annotation, if one exists.
    pub fn get_message(&self) -> Option<&str> {
        self.message.as_ref().map(|m| m.as_str())
    }

    /// Returns the `Span` associated with this annotation.
    pub fn get_span(&self) -> &Span {
        &self.span
    }

    /// Sets the span on this annotation.
    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    /// Returns the tags associated with this annotation.
    pub fn get_tags(&self) -> &[DiagnosticTag] {
        &self.tags
    }

    /// Attaches this tag to this annotation.
    ///
    /// It will not replace any existing tags.
    pub fn tag(mut self, tag: DiagnosticTag) -> Annotation {
        self.tags.push(tag);
        self
    }

    /// Attaches an additional tag to this annotation.
    pub fn push_tag(&mut self, tag: DiagnosticTag) {
        self.tags.push(tag);
    }

    /// Set whether or not the snippet on this annotation should be suppressed when rendering.
    ///
    /// Such annotations are only rendered with their file name and range, if available. This is
    /// intended for backwards compatibility with Ruff diagnostics, which historically used
    /// `TextRange::default` to indicate a file-level diagnostic. In the new diagnostic model, a
    /// [`Span`] with a range of `None` should be used instead, as mentioned in the `Span`
    /// documentation.
    ///
    /// TODO(brent) update this usage in Ruff and remove `is_file_level` entirely. See
    /// <https://github.com/astral-sh/ruff/issues/19688>, especially my first comment, for more
    /// details. As of 2025-09-26 we also use this to suppress snippet rendering for formatter
    /// diagnostics, which also need to have a range, so we probably can't eliminate this entirely.
    pub fn hide_snippet(&mut self, yes: bool) {
        self.hide_snippet = yes;
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
}

/// Tags that can be associated with an annotation.
///
/// These tags are used to provide additional information about the annotation.
/// and are passed through to the language server protocol.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DiagnosticTag {
    /// Unused or unnecessary code. Used for unused parameters, unreachable code, etc.
    Unnecessary,
    /// Deprecated or obsolete code.
    Deprecated,
}

/// A string identifier for a lint rule.
///
/// This string is used in command line and configuration interfaces. The name should always
/// be in kebab case, e.g. `no-foo` (all lower case).
///
/// Rules use kebab case, e.g. `no-foo`.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct LintName(&'static str);

impl LintName {
    pub const fn of(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl std::ops::Deref for LintName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::fmt::Display for LintName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl PartialEq<str> for LintName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for LintName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Uniquely identifies the kind of a diagnostic.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum DiagnosticId {
    Panic,

    /// Some I/O operation failed
    Io,

    /// Some code contains a syntax error
    InvalidSyntax,

    /// A lint violation.
    ///
    /// Lints can be suppressed and some lints can be enabled or disabled in the configuration.
    Lint(Rule),

    /// No rule with the given name exists.
    UnknownRule,

    /// A glob pattern doesn't follow the expected syntax.
    InvalidGlob,

    /// An `include` glob without any patterns.
    ///
    /// ## Why is this bad?
    /// An `include` glob without any patterns won't match any files. This is probably a mistake and
    /// either the `include` should be removed or a pattern should be added.
    ///
    /// ## Example
    /// ```toml
    /// [src]
    /// include = []
    /// ```
    ///
    /// Use instead:
    ///
    /// ```toml
    /// [src]
    /// include = ["src"]
    /// ```
    ///
    /// or remove the `include` option.
    EmptyInclude,

    /// An override configuration is unnecessary because it applies to all files.
    ///
    /// ## Why is this bad?
    /// An overrides section that applies to all files is probably a mistake and can be rolled-up into the root configuration.
    ///
    /// ## Example
    /// ```toml
    /// [[overrides]]
    /// [overrides.rules]
    /// unused-reference = "ignore"
    /// ```
    ///
    /// Use instead:
    ///
    /// ```toml
    /// [rules]
    /// unused-reference = "ignore"
    /// ```
    ///
    /// or
    ///
    /// ```toml
    /// [[overrides]]
    /// include = ["test"]
    ///
    /// [overrides.rules]
    /// unused-reference = "ignore"
    /// ```
    UnnecessaryOverridesSection,

    /// An `overrides` section in the configuration that doesn't contain any overrides.
    ///
    /// ## Why is this bad?
    /// An `overrides` section without any configuration overrides is probably a mistake.
    /// It is either a leftover after removing overrides, or a user forgot to add any overrides,
    /// or used an incorrect syntax to do so (e.g. used `rules` instead of `overrides.rules`).
    ///
    /// ## Example
    /// ```toml
    /// [[overrides]]
    /// include = ["test"]
    /// # no `[overrides.rules]`
    /// ```
    UselessOverridesSection,

    /// Use of a deprecated setting.
    DeprecatedSetting,

    /// The code needs to be formatted.
    Unformatted,

    /// Use of an invalid command-line option.
    InvalidCliOption,

    /// Experimental feature requires preview mode.
    PreviewFeature,

    /// An internal assumption was violated.
    ///
    /// This indicates a bug in the program rather than a user error.
    InternalError,
}

impl DiagnosticId {
    /// Creates a new `DiagnosticId` for a lint.
    pub const fn lint(rule: Rule) -> Self {
        Self::Lint(rule)
    }

    pub fn strip_category(code: &str) -> Option<&str> {
        code.split_once(':').map(|(_, rest)| rest)
    }

    /// Returns a concise description of this diagnostic ID.
    ///
    /// Note that this doesn't include the lint's category. It
    /// only includes the lint's name.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticId::Panic => "panic",
            DiagnosticId::Io => "io",
            DiagnosticId::InvalidSyntax => "invalid-syntax",
            DiagnosticId::Lint(rule) => rule.into(),
            DiagnosticId::UnknownRule => "unknown-rule",
            DiagnosticId::InvalidGlob => "invalid-glob",
            DiagnosticId::EmptyInclude => "empty-include",
            DiagnosticId::UnnecessaryOverridesSection => "unnecessary-overrides-section",
            DiagnosticId::UselessOverridesSection => "useless-overrides-section",
            DiagnosticId::DeprecatedSetting => "deprecated-setting",
            DiagnosticId::Unformatted => "unformatted",
            DiagnosticId::InvalidCliOption => "invalid-cli-option",
            DiagnosticId::PreviewFeature => "preview-feature",
            DiagnosticId::InternalError => "internal-error",
        }
    }

    pub fn is_invalid_syntax(&self) -> bool {
        matches!(self, Self::InvalidSyntax)
    }
}

impl std::fmt::Display for DiagnosticId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A span represents the source of a diagnostic.
///
/// It consists of a `File` and an optional range into that file. When the
/// range isn't present, it semantically implies that the diagnostic refers to
/// the entire file. For example, when the file should be executable but isn't.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    file: SourceFile,
    range: Option<TextRange>,
}

impl Span {
    /// Returns the `SourceFile` attached to this `Span`.
    pub fn file(&self) -> &SourceFile {
        &self.file
    }

    /// Returns the range, if available, attached to this `Span`.
    ///
    /// When there is no range, it is convention to assume that this `Span`
    /// refers to the corresponding `File` as a whole. In some cases, consumers
    /// of this API may use the range `0..0` to represent this case.
    pub fn range(&self) -> Option<TextRange> {
        self.range
    }

    /// Returns a new `Span` with the given `range` attached to it.
    pub fn with_range<R: TextRanged>(self, range: R) -> Span {
        self.with_optional_range(Some(range.textrange()))
    }

    /// Returns a new `Span` with the given optional `range` attached to it.
    pub fn with_optional_range(self, range: Option<TextRange>) -> Span {
        Span { range, ..self }
    }
}

impl From<SourceFile> for Span {
    fn from(file: SourceFile) -> Self {
        Span { file, range: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl Severity {
    fn to_annotate(self) -> AnnotateLevel {
        match self {
            Severity::Info => AnnotateLevel::Info,
            Severity::Warning => AnnotateLevel::Warning,
            Severity::Error => AnnotateLevel::Error,
            // NOTE: Should we really collapse this to "error"?
            //
            // After collapsing this, the snapshot tests seem to reveal that we
            // don't currently have any *tests* with a `fatal` severity level.
            // And maybe *rendering* this as just an `error` is fine. If we
            // really do need different rendering, then I think we can add a
            // `Level::Fatal`. ---AG
            Severity::Fatal => AnnotateLevel::Error,
        }
    }

    pub const fn is_fatal(self) -> bool {
        matches!(self, Severity::Fatal)
    }
}

/// Like [`Severity`] but exclusively for sub-diagnostics.
///
/// This type only exists to add an additional `Help` severity that isn't present in `Severity` or
/// used for main diagnostics. If we want to add `Severity::Help` in the future, this type could be
/// deleted and the two combined again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum SubDiagnosticSeverity {
    Help,
    Info,
    Warning,
    Error,
    Fatal,
}

impl SubDiagnosticSeverity {
    fn to_annotate(self) -> AnnotateLevel {
        match self {
            SubDiagnosticSeverity::Help => AnnotateLevel::Help,
            SubDiagnosticSeverity::Info => AnnotateLevel::Info,
            SubDiagnosticSeverity::Warning => AnnotateLevel::Warning,
            SubDiagnosticSeverity::Error => AnnotateLevel::Error,
            SubDiagnosticSeverity::Fatal => AnnotateLevel::Error,
        }
    }
}

impl Display for SubDiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SubDiagnosticSeverity::Help => "help",
            SubDiagnosticSeverity::Info => "info",
            SubDiagnosticSeverity::Warning => "warning",
            SubDiagnosticSeverity::Error => "error",
            SubDiagnosticSeverity::Fatal => "fatal",
        };
        f.write_str(s)
    }
}

/// Configuration for rendering diagnostics.
#[derive(Clone, Debug)]
pub struct DisplayDiagnosticConfig {
    /// The format to use for diagnostic rendering.
    ///
    /// This uses the "full" format by default.
    format: OutputFormat,
    /// Whether to enable colors or not.
    ///
    /// Disabled by default.
    color: bool,
    /// The number of non-empty lines to show around each snippet.
    ///
    /// NOTE: It seems like making this a property of rendering *could*
    /// be wrong. In particular, I have a suspicion that we may want
    /// more granular control over this, perhaps based on the kind of
    /// diagnostic or even the snippet itself. But I chose to put this
    /// here for now as the most "sensible" place for it to live until
    /// we had more concrete use cases. ---AG
    context: usize,
    /// The "merge window" for annotations.
    ///
    /// If two annotations have fewer than this number of lines between them,
    /// they will be merged into a single annotation.
    merge_window: usize,
    /// Whether to use preview formatting for Ruff diagnostics.
    preview: bool,
    /// Whether to hide the real `Severity` of diagnostics.
    ///
    /// This is intended for temporary use by Ruff, which only has a single `error` severity at the
    /// moment. We should be able to remove this option when Ruff gets more severities.
    hide_severity: bool,
    /// Whether to show the availability of a fix in a diagnostic.
    show_fix_status: bool,
    /// Whether to show the diff for an available fix after the main diagnostic.
    ///
    /// This currently only applies to `OutputFormat::Full`.
    show_fix_diff: bool,
    /// The lowest applicability that should be shown when reporting diagnostics.
    fix_applicability: Applicability,
}

impl Default for DisplayDiagnosticConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayDiagnosticConfig {
    pub fn new() -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            format: OutputFormat::default(),
            color: false,
            context: 2,
            merge_window: 2,
            preview: false,
            hide_severity: false,
            show_fix_status: false,
            show_fix_diff: false,
            fix_applicability: Applicability::Safe,
        }
    }

    /// Whether to enable concise diagnostic output or not.
    pub fn format(self, format: OutputFormat) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig { format, ..self }
    }

    /// Whether to enable colors or not.
    pub fn color(self, yes: bool) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig { color: yes, ..self }
    }

    /// Set the number of contextual lines to show around each snippet.
    pub fn context(self, lines: usize) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            context: lines,
            ..self
        }
    }

    /// Set the "merge window" for annotations.
    ///
    /// If two annotations have fewer than this number of lines between them,
    /// they will be merged into a single annotation.
    pub fn merge_window(self, lines: usize) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            merge_window: lines,
            ..self
        }
    }

    /// Whether to enable preview behavior or not.
    pub fn preview(self, yes: bool) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            preview: yes,
            ..self
        }
    }

    /// Whether to hide a diagnostic's severity or not.
    pub fn hide_severity(self, yes: bool) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            hide_severity: yes,
            ..self
        }
    }

    /// Whether to show a fix's availability or not.
    pub fn with_show_fix_status(self, yes: bool) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            show_fix_status: yes,
            ..self
        }
    }

    /// Whether to show a diff for an available fix after the main diagnostic.
    pub fn show_fix_diff(self, yes: bool) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            show_fix_diff: yes,
            ..self
        }
    }

    /// Set the lowest fix applicability that should be shown.
    ///
    /// In other words, an applicability of `Safe` (the default) would suppress showing fixes or fix
    /// availability for unsafe or display-only fixes.
    ///
    /// Note that this option is currently ignored when `hide_severity` is false.
    pub fn with_fix_applicability(self, applicability: Applicability) -> DisplayDiagnosticConfig {
        DisplayDiagnosticConfig {
            fix_applicability: applicability,
            ..self
        }
    }

    pub fn show_fix_status(&self) -> bool {
        self.show_fix_status
    }

    pub fn fix_applicability(&self) -> Applicability {
        self.fix_applicability
    }
}

/// A representation of the kinds of messages inside a diagnostic.
pub enum ConciseMessage<'a> {
    /// A diagnostic contains a non-empty main message and an empty
    /// primary annotation message.
    MainDiagnostic(&'a str),
    /// A diagnostic contains a non-empty main message and a non-empty
    /// primary annotation message.
    Both { main: &'a str, annotation: &'a str },
    /// A custom concise message has been provided.
    Custom(&'a str),
}

impl<'a> ConciseMessage<'a> {
    pub fn to_str(&self) -> Cow<'a, str> {
        match self {
            ConciseMessage::MainDiagnostic(s) | ConciseMessage::Custom(s) => Cow::Borrowed(s),
            ConciseMessage::Both { .. } => Cow::Owned(self.to_string()),
        }
    }
}

impl std::fmt::Display for ConciseMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ConciseMessage::MainDiagnostic(main) => {
                write!(f, "{main}")
            }
            ConciseMessage::Both { main, annotation } => {
                write!(f, "{main}: {annotation}")
            }
            ConciseMessage::Custom(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Serialize for ConciseMessage<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// A diagnostic message string.
///
/// This is, for all intents and purposes, equivalent to a `Box<str>`.
/// But it does not implement `std::fmt::Display`. Indeed, that it its
/// entire reason for existence. It provides a way to pass a string
/// directly into diagnostic methods that accept messages without copying
/// that string. This works via the `IntoDiagnosticMessage` trait.
///
/// In most cases, callers shouldn't need to use this. Instead, there is
/// a blanket trait implementation for `IntoDiagnosticMessage` for
/// anything that implements `std::fmt::Display`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DiagnosticMessage(Box<str>);

impl DiagnosticMessage {
    /// Returns this message as a borrowed string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DiagnosticMessage {
    fn from(s: &str) -> DiagnosticMessage {
        DiagnosticMessage(s.into())
    }
}

impl From<String> for DiagnosticMessage {
    fn from(s: String) -> DiagnosticMessage {
        DiagnosticMessage(s.into())
    }
}

impl From<Box<str>> for DiagnosticMessage {
    fn from(s: Box<str>) -> DiagnosticMessage {
        DiagnosticMessage(s)
    }
}

impl IntoDiagnosticMessage for DiagnosticMessage {
    fn into_diagnostic_message(self) -> DiagnosticMessage {
        self
    }
}

/// A trait for values that can be converted into a diagnostic message.
///
/// Users of the diagnostic API can largely think of this trait as effectively
/// equivalent to `std::fmt::Display`. Indeed, everything that implements
/// `Display` also implements this trait. That means wherever this trait is
/// accepted, you can use things like `format_args!`.
///
/// The purpose of this trait is to provide a means to give arguments _other_
/// than `std::fmt::Display` trait implementations. Or rather, to permit
/// the diagnostic API to treat them differently. For example, this lets
/// callers wrap a string in a `DiagnosticMessage` and provide it directly
/// to any of the diagnostic APIs that accept a message. This will move the
/// string and avoid any unnecessary copies. (If we instead required only
/// `std::fmt::Display`, then this would potentially result in a copy via the
/// `ToString` trait implementation.)
pub trait IntoDiagnosticMessage {
    fn into_diagnostic_message(self) -> DiagnosticMessage;
}

/// Every `IntoDiagnosticMessage` is accepted, so to is `std::fmt::Display`.
impl<T: std::fmt::Display> IntoDiagnosticMessage for T {
    fn into_diagnostic_message(self) -> DiagnosticMessage {
        DiagnosticMessage::from(self.to_string())
    }
}

/// A secondary identifier for a lint diagnostic.
///
/// For Ruff rules this means the noqa code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize)]
#[serde(transparent)]
pub struct SecondaryCode(String);

impl SecondaryCode {
    pub fn new(code: String) -> Self {
        Self(code)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SecondaryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for SecondaryCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<&str> for SecondaryCode {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<SecondaryCode> for &str {
    fn eq(&self, other: &SecondaryCode) -> bool {
        other.eq(self)
    }
}

// for `hashbrown::EntryRef`
impl From<&SecondaryCode> for SecondaryCode {
    fn from(value: &SecondaryCode) -> Self {
        value.clone()
    }
}

pub fn create_lint_diagnostic<B, S>(
    body: B,
    suggestion: Option<S>,
    range: TextRange,
    fix: Option<Fix>,
    file: SourceFile,
    rule: Rule,
) -> Diagnostic
where
    B: Display,
    S: Display,
{
    let mut diagnostic = Diagnostic::new(DiagnosticId::Lint(rule), Severity::Error, body);

    let span = Span::from(file).with_range(range);
    let mut annotation = Annotation::primary(span);
    // The `0..0` range is used to highlight file-level diagnostics.
    //
    // TODO(brent) We should instead set this flag on annotations for individual lint rules that
    // actually need it, but we need to be able to cache the new diagnostic model first. See
    // https://github.com/astral-sh/ruff/issues/19688.
    if range == TextRange::default() {
        annotation.hide_snippet(true);
    }
    diagnostic.annotate(annotation);

    if let Some(suggestion) = suggestion {
        diagnostic.help(suggestion);
    }

    if let Some(fix) = fix {
        diagnostic.set_fix(fix);
    }

    diagnostic.set_secondary_code(SecondaryCode::new(rule.noqa_code().to_string()));
    diagnostic.set_documentation_url(Some(format!(
        "{}/en/stable/rules/{}",
        env!("CARGO_PKG_HOMEPAGE"),
        rule.name()
    )));

    diagnostic
}
