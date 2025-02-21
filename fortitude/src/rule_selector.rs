/// Utilities for selecting groups of rules
// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use globset::{Glob, GlobMatcher};
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::registry::{Category, Rule, RuleNamespace, RuleSet};
use crate::rule_redirects::{get_deprecated_category, get_redirect};
use crate::rules::{RuleCodePrefix, RuleGroup, RuleIter};
use crate::settings::{PatternPrefixPair, PreviewMode};
use crate::{display_settings, fs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleSelector {
    /// Select all rules (includes rules in preview if enabled)
    All,
    /// Select all rules for a given category.
    Category(Category),
    /// Select all rules for a given category with a given prefix.
    Prefix {
        prefix: RuleCodePrefix,
        redirected_from: Option<&'static str>,
    },
    /// Select an individual rule with a given prefix.
    Rule {
        prefix: RuleCodePrefix,
        redirected_from: Option<&'static str>,
    },
    /// Select rules from a removed category
    DeprecatedCategory {
        rules: Vec<&'static str>,
        redirected_to: Vec<RuleCodePrefix>,
        redirected_from: String,
    },
}

impl From<Category> for RuleSelector {
    fn from(category: Category) -> Self {
        Self::Category(category)
    }
}

impl Ord for RuleSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO(zanieb): We ought to put "ALL" and "Category" selectors
        // above those that are rule specific but it's not critical for now
        self.prefix_and_code().cmp(&other.prefix_and_code())
    }
}

impl PartialOrd for RuleSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for RuleSelector {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // **Changes should be reflected in `parse_no_redirect` as well**
        match s {
            "ALL" => Ok(Self::All),
            _ => {
                let (s, redirected_from) = match get_redirect(s) {
                    Some((from, target)) => (target, Some(from)),
                    None => (s, None),
                };

                // If passed full name of rule, use the equivalent code instead.
                if let Ok(rule) = Rule::from_str(s) {
                    let c = rule.noqa_code().to_string();
                    return Self::from_str(c.as_str());
                }

                // If passed a deprecated category, get list of all rules in that category.
                if let Some((rules, redirects)) = get_deprecated_category(s) {
                    let redirected_to = redirects
                        .iter()
                        .map(|code| {
                            let (category, code) = Category::parse_code(code).unwrap();
                            RuleCodePrefix::parse(&category, code).unwrap()
                        })
                        .collect();
                    return Ok(Self::DeprecatedCategory {
                        rules,
                        redirected_to,
                        redirected_from: s.to_string(),
                    });
                }

                let (category, code) =
                    Category::parse_code(s).ok_or_else(|| ParseError::Unknown(s.to_string()))?;

                if code.is_empty() {
                    return Ok(Self::Category(category));
                }

                let prefix = RuleCodePrefix::parse(&category, code)
                    .map_err(|_| ParseError::Unknown(s.to_string()))?;

                if is_single_rule_selector(&prefix) {
                    Ok(Self::Rule {
                        prefix,
                        redirected_from,
                    })
                } else {
                    Ok(Self::Prefix {
                        prefix,
                        redirected_from,
                    })
                }
            }
        }
    }
}

/// Returns `true` if the [`RuleCodePrefix`] matches a single rule exactly
/// (e.g., `E225`, as opposed to `E2`).
pub(crate) fn is_single_rule_selector(prefix: &RuleCodePrefix) -> bool {
    let mut rules = prefix.rules();

    // The selector must match a single rule.
    let Some(rule) = rules.next() else {
        return false;
    };
    if rules.next().is_some() {
        return false;
    }

    // The rule must match the selector exactly.
    rule.noqa_code().suffix() == prefix.short_code()
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown rule selector: `{0}`")]
    // TODO(martin): tell the user how to discover rule codes via the CLI once such a command is
    // implemented (but that should of course be done only in ruff and not here)
    Unknown(String),
}

impl RuleSelector {
    pub fn prefix_and_code(&self) -> (String, String) {
        match self {
            RuleSelector::All => ("".to_string(), "ALL".to_string()),
            RuleSelector::Prefix { prefix, .. } | RuleSelector::Rule { prefix, .. } => (
                prefix.category().common_prefix().to_string(),
                prefix.short_code().to_string(),
            ),
            RuleSelector::Category(l) => (l.common_prefix().to_string(), "".to_string()),
            RuleSelector::DeprecatedCategory {
                redirected_from, ..
            } => (redirected_from.to_string(), "".to_string()),
        }
    }
}

impl Serialize for RuleSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (prefix, code) = self.prefix_and_code();
        serializer.serialize_str(&format!("{prefix}{code}"))
    }
}

impl<'de> Deserialize<'de> for RuleSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // We are not simply doing:
        // let s: &str = Deserialize::deserialize(deserializer)?;
        // FromStr::from_str(s).map_err(de::Error::custom)
        // here because the toml crate apparently doesn't support that
        // (as of toml v0.6.0 running `cargo test` failed with the above two lines)
        deserializer.deserialize_str(SelectorVisitor)
    }
}

struct SelectorVisitor;

impl Visitor<'_> for SelectorVisitor {
    type Value = RuleSelector;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "expected a string code identifying a category or specific rule, or a partial rule code or ALL to refer to all rules",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        FromStr::from_str(v).map_err(de::Error::custom)
    }
}

impl RuleSelector {
    /// Return all matching rules, regardless of rule group filters like preview and deprecated.
    pub fn all_rules(&self) -> impl Iterator<Item = Rule> + '_ {
        match self {
            RuleSelector::All => RuleSelectorIter::All(Rule::iter()),
            RuleSelector::Category(category) => RuleSelectorIter::Vec(category.rules()),
            RuleSelector::Prefix { prefix, .. } | RuleSelector::Rule { prefix, .. } => {
                RuleSelectorIter::Vec(prefix.clone().rules())
            }
            RuleSelector::DeprecatedCategory { redirected_to, .. } => RuleSelectorIter::Vec(
                redirected_to
                    .iter()
                    .flat_map(|prefix| prefix.clone().rules())
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }

    /// Returns rules matching the selector, taking into account rule groups like preview and deprecated.
    pub fn rules<'a>(&'a self, preview: &PreviewOptions) -> impl Iterator<Item = Rule> + 'a {
        let preview_enabled = preview.mode.is_enabled();
        let preview_require_explicit = preview.require_explicit;

        self.all_rules().filter(move |rule| {
            match rule.group() {
                // Always include stable rules
                RuleGroup::Stable => true,
                // Enabling preview includes all preview rules unless explicit selection is turned on
                RuleGroup::Preview => {
                    preview_enabled && (self.is_exact() || !preview_require_explicit)
                }
                // Deprecated rules are excluded in preview mode and with 'All' option unless explicitly selected
                RuleGroup::Deprecated => {
                    (!preview_enabled || self.is_exact())
                        && !matches!(self, RuleSelector::All { .. })
                }
                // Removed rules are included if explicitly selected but will error downstream
                RuleGroup::Removed => self.is_exact(),
            }
        })
    }

    /// Returns true if this selector is exact i.e. selects a single rule by code
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Rule { .. })
    }
}

pub enum RuleSelectorIter {
    All(RuleIter),
    Vec(std::vec::IntoIter<Rule>),
}

impl Iterator for RuleSelectorIter {
    type Item = Rule;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RuleSelectorIter::All(iter) => iter.next(),
            RuleSelectorIter::Vec(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreviewOptions {
    pub mode: PreviewMode,
    /// If true, preview rule selection requires explicit codes e.g. not prefixes.
    /// Generally this should be derived from the user-facing `explicit-preview-rules` option.
    pub require_explicit: bool,
}

impl RuleSelector {
    pub fn specificity(&self) -> Specificity {
        match self {
            RuleSelector::All => Specificity::All,
            RuleSelector::Category(..) => Specificity::Category,
            RuleSelector::Rule { .. } => Specificity::Rule,
            RuleSelector::Prefix { prefix, .. } => {
                let prefix: &'static str = prefix.short_code();
                match prefix.len() {
                    1 => Specificity::Prefix1Char,
                    2 => Specificity::Prefix2Chars,
                    3 => Specificity::Prefix3Chars,
                    4 => Specificity::Prefix4Chars,
                    _ => panic!("RuleSelector::specificity doesn't yet support codes with so many characters"),
                }
            }
            RuleSelector::DeprecatedCategory {
                redirected_from, ..
            } => {
                let prefix = redirected_from
                    .chars()
                    .filter(|c| c.is_numeric())
                    .collect::<String>();
                match prefix.len() {
                    0 => Specificity::Category,
                    1 => Specificity::Prefix1Char,
                    2 => Specificity::Prefix2Chars,
                    3 => Specificity::Rule,
                    _ => panic!(
                        "Deprecated category rule codes should have exactly 3 numerical characters"
                    ),
                }
            }
        }
    }

    /// Parse [`RuleSelector`] from a string; but do not follow redirects.
    pub fn parse_no_redirect(s: &str) -> Result<Self, ParseError> {
        // **Changes should be reflected in `from_str` as well**
        match s {
            "ALL" => Ok(Self::All),
            _ => {
                let (category, code) =
                    Category::parse_code(s).ok_or_else(|| ParseError::Unknown(s.to_string()))?;

                if code.is_empty() {
                    return Ok(Self::Category(category));
                }

                let prefix = RuleCodePrefix::parse(&category, code)
                    .map_err(|_| ParseError::Unknown(s.to_string()))?;

                if is_single_rule_selector(&prefix) {
                    Ok(Self::Rule {
                        prefix,
                        redirected_from: None,
                    })
                } else {
                    Ok(Self::Prefix {
                        prefix,
                        redirected_from: None,
                    })
                }
            }
        }
    }
}

#[derive(EnumIter, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum Specificity {
    /// The specificity when selecting all rules (e.g., `--select ALL`).
    All,
    /// The specificity when selecting a category (e.g., `--select PLE` or `--select UP`).
    Category,
    /// The specificity when selecting via a rule prefix with a one-character code (e.g., `--select PLE1`).
    Prefix1Char,
    /// The specificity when selecting via a rule prefix with a two-character code (e.g., `--select PLE12`).
    Prefix2Chars,
    /// The specificity when selecting via a rule prefix with a three-character code (e.g., `--select PLE123`).
    Prefix3Chars,
    /// The specificity when selecting via a rule prefix with a four-character code (e.g., `--select PLE1234`).
    Prefix4Chars,
    /// The specificity when selecting an individual rule (e.g., `--select PLE1205`).
    Rule,
}

pub mod clap_completion {
    use clap::builder::{PossibleValue, TypedValueParser, ValueParserFactory};
    use strum::IntoEnumIterator;

    use crate::{
        registry::{Category, RuleNamespace},
        rule_selector::is_single_rule_selector,
        rule_selector::RuleSelector,
        rules::RuleCodePrefix,
    };

    #[derive(Clone)]
    pub struct RuleSelectorParser;

    impl ValueParserFactory for RuleSelector {
        type Parser = RuleSelectorParser;

        fn value_parser() -> Self::Parser {
            RuleSelectorParser
        }
    }

    impl TypedValueParser for RuleSelectorParser {
        type Value = RuleSelector;

        fn parse_ref(
            &self,
            cmd: &clap::Command,
            arg: Option<&clap::Arg>,
            value: &std::ffi::OsStr,
        ) -> Result<Self::Value, clap::Error> {
            let value = value
                .to_str()
                .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;

            value.parse().map_err(|_| {
                let mut error =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    error.insert(
                        clap::error::ContextKind::InvalidArg,
                        clap::error::ContextValue::String(arg.to_string()),
                    );
                }
                error.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(value.to_string()),
                );
                error
            })
        }

        fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
            Some(Box::new(
                std::iter::once(PossibleValue::new("ALL").help("all rules")).chain(
                    Category::iter()
                        .filter_map(|l| {
                            let prefix = l.common_prefix();
                            (!prefix.is_empty()).then(|| PossibleValue::new(prefix).help(l.name()))
                        })
                        .chain(RuleCodePrefix::iter().filter_map(|prefix| {
                            // Ex) `UP`
                            if prefix.short_code().is_empty() {
                                let code = prefix.category().common_prefix();
                                let name = prefix.category().name();
                                return Some(PossibleValue::new(code).help(name));
                            }

                            // Ex) `UP004`
                            if is_single_rule_selector(&prefix) {
                                let rule = prefix.rules().next()?;
                                let code = format!(
                                    "{}{}",
                                    prefix.category().common_prefix(),
                                    prefix.short_code()
                                );
                                let name: &'static str = rule.into();
                                return Some(PossibleValue::new(code).help(name));
                            }

                            None
                        })),
                ),
            ))
        }
    }
}

#[derive(Debug)]
pub struct PerFileIgnore {
    basename: String,
    absolute: PathBuf,
    negated: bool,
    rules: RuleSet,
}

impl PerFileIgnore {
    pub fn new(
        mut pattern: String,
        prefixes: &[RuleSelector],
        project_root: Option<&Path>,
    ) -> Self {
        // Rules in preview are included here even if preview mode is disabled; it's
        // safe to ignore disabled rules
        let rules: RuleSet = prefixes.iter().flat_map(RuleSelector::all_rules).collect();
        let negated = pattern.starts_with('!');
        if negated {
            pattern.drain(..1);
        }
        let path = Path::new(&pattern);
        let absolute = match project_root {
            Some(project_root) => fs::normalize_path_to(path, project_root),
            None => fs::normalize_path(path),
        };

        Self {
            basename: pattern,
            absolute,
            negated,
            rules,
        }
    }
}

/// Convert a list of `PatternPrefixPair` structs to `PerFileIgnore`.
pub fn collect_per_file_ignores(pairs: Vec<PatternPrefixPair>) -> Vec<PerFileIgnore> {
    let mut per_file_ignores: HashMap<String, Vec<RuleSelector>> = HashMap::new();
    for pair in pairs {
        per_file_ignores
            .entry(pair.pattern)
            .or_default()
            .push(pair.prefix);
    }
    per_file_ignores
        .into_iter()
        .map(|(pattern, prefixes)| PerFileIgnore::new(pattern, &prefixes, None))
        .collect()
}

#[derive(Debug, Clone)]
pub struct CompiledPerFileIgnore {
    pub absolute_matcher: GlobMatcher,
    pub basename_matcher: GlobMatcher,
    pub negated: bool,
    pub rules: RuleSet,
}

impl Display for CompiledPerFileIgnore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        display_settings! {
            formatter = f,
            fields = [
                self.absolute_matcher | globmatcher,
                self.basename_matcher | globmatcher,
                self.negated,
                self.rules,
            ]
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompiledPerFileIgnoreList {
    // Ordered as (absolute path matcher, basename matcher, rules)
    ignores: Vec<CompiledPerFileIgnore>,
}

impl CompiledPerFileIgnoreList {
    /// Given a list of patterns, create a `GlobSet`.
    pub fn resolve(per_file_ignores: Vec<PerFileIgnore>) -> anyhow::Result<Self> {
        let ignores: anyhow::Result<Vec<_>> = per_file_ignores
            .into_iter()
            .map(|per_file_ignore| {
                // Construct absolute path matcher.
                let absolute_matcher =
                    Glob::new(&per_file_ignore.absolute.to_string_lossy())?.compile_matcher();

                // Construct basename matcher.
                let basename_matcher = Glob::new(&per_file_ignore.basename)?.compile_matcher();

                Ok(CompiledPerFileIgnore {
                    absolute_matcher,
                    basename_matcher,
                    negated: per_file_ignore.negated,
                    rules: per_file_ignore.rules,
                })
            })
            .collect();
        Ok(Self { ignores: ignores? })
    }
}

impl Display for CompiledPerFileIgnoreList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.ignores.is_empty() {
            write!(f, "{{}}")?;
        } else {
            writeln!(f, "{{")?;
            for ignore in &self.ignores {
                writeln!(f, "\t{ignore}")?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl Deref for CompiledPerFileIgnoreList {
    type Target = Vec<CompiledPerFileIgnore>;

    fn deref(&self) -> &Self::Target {
        &self.ignores
    }
}
