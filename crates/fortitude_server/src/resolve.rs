// Adapted from ruff
// Copyright 2022-2025 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::Path;

use fortitude_linter::settings::{CheckSettings, FileResolverSettings};
use fortitude_workspace::resolver::{match_any_exclusion, match_any_inclusion};

use crate::edit::LanguageId;

/// Return `true` if the document at the given [`Path`] should be excluded from linting.
pub(crate) fn is_document_excluded_for_linting(
    path: &Path,
    resolver_settings: &FileResolverSettings,
    linter_settings: &CheckSettings,
    language_id: Option<LanguageId>,
) -> bool {
    is_document_excluded(path, resolver_settings, Some(linter_settings), language_id)
}

/// Return `true` if the document at the given [`Path`] should be excluded.
///
/// The tool-specific settings should be provided if the request for the document is specific to
/// that tool. For example, a diagnostics request should provide the linter settings while the
/// formatting request should provide the formatter settings.
///
/// The logic for the resolution considers both inclusion and exclusion and is as follows:
/// 1. Check for global `exclude` and `extend-exclude` options along with tool specific `exclude`
///    option (`check.exclude`, `format.exclude`)
/// 2. Check for global `include` and `extend-include` options.
/// 3. Check if the language ID is Fortran, in which case the document is included.
/// 4. If none of the above conditions are met, the document is excluded.
fn is_document_excluded(
    path: &Path,
    resolver_settings: &FileResolverSettings,
    _linter_settings: Option<&CheckSettings>,
    language_id: Option<LanguageId>,
) -> bool {
    // TODO(peter): pass linter and formatter settings when they're implemented
    if let Some(exclusion) = match_any_exclusion(path, resolver_settings, None, None) {
        tracing::debug!("Ignored path via `{}`: {}", exclusion, path.display());
        return true;
    }

    if let Some(inclusion) = match_any_inclusion(path, resolver_settings) {
        tracing::debug!("Included path via `{}`: {}", inclusion, path.display());
        false
    } else if let Some(LanguageId::Fortran) = language_id {
        tracing::debug!("Included path via Fortran language ID: {}", path.display());
        false
    } else {
        tracing::debug!(
            "Ignored path as it's not in the inclusion set: {}",
            path.display()
        );
        true
    }
}
