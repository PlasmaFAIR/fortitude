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

static REDIRECTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("B001", "C001"),
        ("B011", "C011"),
        ("P001", "C021"),
        ("P011", "MOD001"),
        ("P021", "C022"),
    ])
});

/// Return the deprecated category and all contained rules if provided with the
/// name of a deprecated category. Otherwise, return None.
pub(crate) fn get_deprecated_category(code: &str) -> Option<(&'static str, &[&'static str])> {
    DEPRECATED_CATEGORIES
        .get_key_value(code)
        .map(|(k, v)| (*k, v.as_slice()))
}

static DEPRECATED_CATEGORIES: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from_iter([
            ("B", vec!["C001", "C011"]),
            ("B0", vec!["C001", "C011"]),
            ("B00", vec!["C001"]),
            ("B01", vec!["C011"]),
            ("bugprone", vec!["C001", "C011"]),
            ("P", vec!["C021", "C022", "MOD001"]),
            ("P0", vec!["C021", "C022", "MOD001"]),
            ("P00", vec!["C021"]),
            ("P01", vec!["MOD001"]),
            ("P02", vec!["C022"]),
            ("precision", vec!["C021", "C022", "MOD001"]),
        ])
    });
