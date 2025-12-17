use crate::server::Result;
use crate::session::{Client, DocumentSnapshot};
use fortitude_linter::registry::Rule;
use lazy_regex::regex;
use lsp_types::{self as types, request as req};
use ruff_diagnostics::FixAvailability;
use ruff_source_file::OneIndexed;
use std::fmt::Write;
use std::str::FromStr;

pub(crate) struct Hover;

impl super::RequestHandler for Hover {
    type RequestType = req::HoverRequest;
}

impl super::BackgroundDocumentRequestHandler for Hover {
    fn document_url(params: &types::HoverParams) -> std::borrow::Cow<'_, lsp_types::Url> {
        std::borrow::Cow::Borrowed(&params.text_document_position_params.text_document.uri)
    }
    fn run_with_snapshot(
        snapshot: DocumentSnapshot,
        _client: &Client,
        params: types::HoverParams,
    ) -> Result<Option<types::Hover>> {
        Ok(hover(&snapshot, &params.text_document_position_params))
    }
}

pub(crate) fn hover(
    snapshot: &DocumentSnapshot,
    position: &types::TextDocumentPositionParams,
) -> Option<types::Hover> {
    // From Ruff: Hover only operates on text documents or notebook cells
    let document = snapshot.query().as_single_document();
    let line_number: usize = position
        .position
        .line
        .try_into()
        .expect("line number should fit within a usize");
    let line_range = document.index().line_range(
        OneIndexed::from_zero_indexed(line_number),
        document.contents(),
    );

    let line = &document.contents()[line_range];

    // Get the list of codes.
    let allow_comment_regex = regex!(r#"! allow\((?P<codes>.*)\)\s*"#);
    let allow_comment_captures = allow_comment_regex.captures(line)?;
    let codes_match = allow_comment_captures.name("codes")?;
    let codes_start = codes_match.start();
    let rule_regex = regex!(r#"\w[-\w\d]*"#);
    let cursor: usize = position
        .position
        .character
        .try_into()
        .expect("column number should fit within a usize");
    let word = rule_regex.find_iter(codes_match.as_str()).find(|code| {
        cursor >= (code.start() + codes_start) && cursor < (code.end() + codes_start)
    })?;

    // Get rule for the code under the cursor.
    let rule = Rule::from_code(word.as_str()).or(Rule::from_str(word.as_str()));

    let output = if let Ok(rule) = rule {
        format_rule_text(rule)
    } else {
        format!("{}: Rule not found", word.as_str())
    };

    let hover = types::Hover {
        contents: types::HoverContents::Markup(types::MarkupContent {
            kind: types::MarkupKind::Markdown,
            value: output,
        }),
        range: None,
    };

    Some(hover)
}

fn format_rule_text(rule: Rule) -> String {
    let mut output = String::new();
    let _ = write!(&mut output, "# {} ({})", rule, rule.noqa_code());
    output.push('\n');
    output.push('\n');

    let fix_availability = rule.fixable();
    if matches!(
        fix_availability,
        FixAvailability::Always | FixAvailability::Sometimes
    ) {
        output.push_str(&fix_availability.to_string());
        output.push('\n');
        output.push('\n');
    }

    if rule.is_preview() {
        output.push_str(r"This rule is in preview and is not stable.");
        output.push('\n');
        output.push('\n');
    }

    if let Some(explanation) = rule.explanation() {
        output.push_str(explanation.trim());
    } else {
        tracing::warn!("Rule {} does not have an explanation", rule.noqa_code());
        output.push_str("An issue occurred: an explanation for this rule was not found.");
    }
    output
}
