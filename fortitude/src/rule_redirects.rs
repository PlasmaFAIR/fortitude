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
        ("F001", "S091"),
        ("R001", "C031"),
        ("IO001", "C041"),
        ("IO011", "C032"),
        ("IO012", "PORT001"),
        ("T001", "C051"),
        ("T002", "C052"),
        ("T003", "S201"),
        ("T004", "C053"),
        ("T011", "PORT011"),
        ("T012", "PORT012"),
        ("T021", "PORT021"),
    ])
});

/// If provided with a rule/category/prefix from a deprecated category,
/// returns all associated rules and redirects. Otherwise returns None.
pub(crate) fn get_deprecated_category(
    code: &str,
) -> Option<(Vec<&'static str>, Vec<&'static str>)> {
    let short_code = match DEPRECATED_CATEGORY_SHORT_NAMES.get(&code) {
        Some(short_code) => short_code,
        None => code,
    };
    let category = short_code.trim_end_matches(|c: char| c.is_numeric());
    if DEPRECATED_CATEGORY_SHORT_NAMES
        .values()
        .all(|&v| v != category)
    {
        return None;
    }
    let (rules, redirects): (Vec<_>, Vec<_>) = REDIRECTS
        .iter()
        .filter_map(|(&k, &v)| {
            if k.starts_with(short_code) {
                Some((k, v))
            } else {
                None
            }
        })
        .unzip();
    if rules.is_empty() {
        return None;
    }
    Some((rules, redirects))
}

static DEPRECATED_CATEGORY_SHORT_NAMES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from_iter([
            ("bugprone", "B"),
            ("precision", "P"),
            ("filesystem", "F"),
            ("readability", "R"),
            ("io", "IO"),
        ])
    });
