use std::borrow::Cow;

use rustc_hash::FxHashMap;

use crate::{
    PositionEncoding,
    edit::{Replacement, ToRangeExt},
    session::DocumentQuery,
};
use fortitude_linter::{FixerResult, ast_entrypoint_map, rules_to_path_rules, rules_to_text_rules};
use ruff_source_file::{LineIndex, SourceFileBuilder};

/// A simultaneous fix made across a single text document or among an arbitrary
/// number of notebook cells.
pub(crate) type Fixes = FxHashMap<lsp_types::Url, Vec<lsp_types::TextEdit>>;

pub(crate) fn fix_all(query: &DocumentQuery, encoding: PositionEncoding) -> crate::Result<Fixes> {
    let source_kind = query.make_source_kind();
    let settings = query.settings();
    let document_path = query.virtual_file_path();

    // TODO(peter): If the document is excluded, return an empty list of diagnostics.

    let file =
        SourceFileBuilder::new(document_path.to_string_lossy(), source_kind.as_str()).finish();

    let rules = &settings.check.rules;
    let path_rules = rules_to_path_rules(rules);
    let text_rules = rules_to_text_rules(rules);
    let ast_entrypoints = ast_entrypoint_map(rules);

    // We need to iteratively apply all safe fixes onto a single file and then
    // create a diff between the modified file and the original source to use as a single workspace
    // edit.
    // If we simply generated the diagnostics with `check_path` and then applied fixes individually,
    // there's a possibility they could overlap or introduce new problems that need to be fixed,
    // which is inconsistent with how `ruff check --fix` works.
    let FixerResult { transformed, .. } = fortitude_linter::check_and_fix_file(
        rules,
        &path_rules,
        &text_rules,
        &ast_entrypoints,
        &document_path,
        &file,
        settings,
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
