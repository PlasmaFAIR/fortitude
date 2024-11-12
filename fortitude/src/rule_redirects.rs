/// Mappings from old rule codes to new ones.
/// Currently just future-proofing!
// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::sync::LazyLock;

/// Returns the redirect target for the given code.
#[allow(dead_code)]
pub(crate) fn get_redirect_target(code: &str) -> Option<&'static str> {
    REDIRECTS.get(code).copied()
}

/// Returns the code and the redirect target if the given code is a redirect.
/// (The same code is returned to obtain it with a static lifetime).
pub(crate) fn get_redirect(code: &str) -> Option<(&'static str, &'static str)> {
    REDIRECTS.get_key_value(code).map(|(k, v)| (*k, *v))
}

static REDIRECTS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| HashMap::from_iter([]));
