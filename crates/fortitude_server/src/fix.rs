use std::borrow::Cow;

use rustc_hash::FxHashMap;

use crate::{
    PositionEncoding,
    edit::{Replacement, ToRangeExt},
    resolve::is_document_excluded_for_linting,
    session::DocumentQuery,
};
use fortitude_linter::FixerResult;
use ruff_source_file::{LineIndex, SourceFileBuilder};

/// A simultaneous fix made across a single text document. In future, this could
/// also be used to support fixes across an arbitrary number of notebook cells.
pub(crate) type Fixes = FxHashMap<lsp_types::Url, Vec<lsp_types::TextEdit>>;

pub(crate) fn fix_all(query: &DocumentQuery, encoding: PositionEncoding) -> crate::Result<Fixes> {
    let source_kind = query.make_source_kind();
    let settings = query.settings();
    let document_path = query.virtual_file_path();

    if is_document_excluded_for_linting(
        &document_path,
        &settings.file_resolver,
        &settings.check,
        query.text_document_language_id(),
    ) {
        return Ok(Fixes::default());
    }

    let file =
        SourceFileBuilder::new(document_path.to_string_lossy(), source_kind.as_str()).finish();

    // We need to iteratively apply all safe fixes onto a single file and then
    // create a diff between the modified file and the original source to use as a single workspace
    // edit.
    // If we simply generated the diagnostics with `check_path` and then applied fixes individually,
    // there's a possibility they could overlap or introduce new problems that need to be fixed,
    // which is inconsistent with how `fortitude check --fix` works.
    let FixerResult { transformed, .. } = fortitude_linter::check_and_fix_file(
        &document_path,
        &file,
        &settings.check,
        fortitude_linter::settings::IgnoreAllowComments::Disabled,
    )?;

    // fast path: if `transformed` is still borrowed, no changes were made and we can return early
    if let Cow::Borrowed(_) = transformed {
        return Ok(Fixes::default());
    }

    let source_index = LineIndex::from_source_text(&source_kind);

    let modified = transformed.source_text();
    let modified_index = LineIndex::from_source_text(modified);

    let Replacement {
        source_range,
        modified_range,
    } = Replacement::between(
        &source_kind,
        source_index.line_starts(),
        modified,
        modified_index.line_starts(),
    );
    Ok([(
        query.make_key().into_url(),
        vec![lsp_types::TextEdit {
            range: source_range.to_range(&source_kind, &source_index, encoding),
            new_text: modified[modified_range].to_owned(),
        }],
    )]
    .into_iter()
    .collect())
}
