pub mod allow_comments;
pub mod ast;
pub mod diagnostic_message;
pub mod diagnostics;
pub mod fix;
pub mod fs;
pub mod locator;
#[macro_use]
pub mod logging;
pub mod message;
pub mod registry;
pub mod rule_redirects;
pub mod rule_selector;
pub mod rule_table;
pub mod rules;
pub mod settings;
#[cfg(test)]
mod test;
pub mod text_helpers;

use allow_comments::{check_allow_comments, gather_allow_comments};
use ast::FortitudeNode;
use diagnostic_message::DiagnosticMessage;
use diagnostics::{Diagnostics, FixMap};
use fix::{fix_file, FixResult};
use locator::Locator;
use registry::AsRule;
use rule_table::RuleTable;
use rules::error::syntax_error::SyntaxError;
#[cfg(any(feature = "test-rules", test))]
use rules::testing::test_rules::{self, TestRule, TEST_RULES};
use rules::Rule;
use rules::{AstRuleEnum, PathRuleEnum, TextRuleEnum};
use settings::{FixMode, Settings};

use anyhow::{anyhow, Context};
use colored::Colorize;
use itertools::Itertools;
use log::warn;
use ruff_diagnostics::{Diagnostic, DiagnosticKind};
use ruff_source_file::SourceFile;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Violation type
// --------------

pub trait FromAstNode {
    fn from_node<T: Into<DiagnosticKind>>(violation: T, node: &Node) -> Self;
}

impl FromAstNode for Diagnostic {
    fn from_node<T: Into<DiagnosticKind>>(violation: T, node: &Node) -> Self {
        Self::new(violation, node.textrange())
    }
}

// Rule trait
// ----------

/// Implemented by rules that act directly on the file path.
pub trait PathRule {
    fn check(settings: &Settings, path: &Path) -> Option<Diagnostic>;
}

/// Implemented by rules that analyse lines of code directly, using regex or otherwise.
pub trait TextRule {
    fn check(settings: &Settings, source: &SourceFile) -> Vec<Diagnostic>;
}

/// Implemented by rules that analyse the abstract syntax tree.
pub trait AstRule {
    fn check(settings: &Settings, node: &Node, source: &SourceFile) -> Option<Vec<Diagnostic>>;

    /// Return list of tree-sitter node types on which a rule should trigger.
    fn entrypoints() -> Vec<&'static str>;
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub fn check_file(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Diagnostics> {
    let (mut messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
            ignore_allow_comments,
        ) {
            if !fixed.is_empty() {
                match fix_mode {
                    FixMode::Apply => {
                        let mut out_file = File::create(path)?;
                        out_file.write_all(transformed.source_text().as_bytes())?;
                    }
                    // TODO: diff
                    FixMode::Diff => {}
                    FixMode::Generate => {}
                }
            }

            (result, fixed)
        } else {
            // Failed to fix, so just lint the original source
            let result = check_only_file(
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                file,
                settings,
                ignore_allow_comments,
            )?;
            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            file,
            settings,
            ignore_allow_comments,
        )?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    // Ignore based on per-file-ignores.
    // If the DiagnosticMessage is discarded, its fix will also be ignored.
    let per_file_ignores = &settings.check.per_file_ignores;
    let per_file_ignores = if !messages.is_empty() && !per_file_ignores.is_empty() {
        fs::ignores_from_path(path, per_file_ignores)
    } else {
        vec![]
    };
    if !per_file_ignores.is_empty() {
        messages.retain(|message| {
            if let Some(rule) = message.rule() {
                !per_file_ignores.contains(&rule)
            } else {
                true
            }
        });
    }

    Ok(Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    })
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub fn check_only_file(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Vec<DiagnosticMessage>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;
    let tree = parser
        .parse(file.source_text(), None)
        .context("Failed to parse")?;

    let violations = check_path(
        rules,
        path_rules,
        text_rules,
        ast_entrypoints,
        path,
        file,
        settings,
        &tree,
        ignore_allow_comments,
    );

    Ok(violations
        .into_iter()
        .map(|v| DiagnosticMessage::from_ruff(file, v))
        .collect_vec())
}

/// Check an already parsed file. This actually does all the checking,
/// `check_only_file`/`check_and_fix_file` wrap this
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_path(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &SourceFile,
    settings: &Settings,
    tree: &Tree,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();
    let mut allow_comments = Vec::new();

    // Check file paths directly
    for rule in path_rules {
        if let Some(violation) = rule.check(settings, path) {
            violations.push(violation);
        }
    }

    // Perform plain text analysis
    for rule in text_rules {
        violations.extend(rule.check(settings, file));
    }

    // Perform AST analysis
    let root = tree.root_node();
    for node in once(root).chain(root.descendants()) {
        if rules.enabled(Rule::SyntaxError) && node.is_missing() {
            violations.push(Diagnostic::from_node(SyntaxError {}, &node));
        }

        if let Some(rules) = ast_entrypoints.get(node.kind()) {
            for rule in rules {
                if let Some(violation) = rule.check(settings, &node, file) {
                    for v in violation {
                        violations.push(v);
                    }
                }
            }
        }
        if let Some(allow_rules) = gather_allow_comments(&node, file) {
            allow_comments.push(allow_rules);
        };
    }

    // Raise violations for internal test rules
    #[cfg(any(feature = "test-rules", test))]
    {
        for test_rule in TEST_RULES {
            if !rules.enabled(*test_rule) {
                continue;
            }
            let diagnostic = match test_rule {
                Rule::StableTestRule => test_rules::StableTestRule::check(),
                Rule::StableTestRuleSafeFix => test_rules::StableTestRuleSafeFix::check(),
                Rule::StableTestRuleUnsafeFix => test_rules::StableTestRuleUnsafeFix::check(),
                Rule::StableTestRuleDisplayOnlyFix => {
                    test_rules::StableTestRuleDisplayOnlyFix::check()
                }
                Rule::PreviewTestRule => test_rules::PreviewTestRule::check(),
                Rule::DeprecatedTestRule => test_rules::DeprecatedTestRule::check(),
                Rule::AnotherDeprecatedTestRule => test_rules::AnotherDeprecatedTestRule::check(),
                Rule::RemovedTestRule => test_rules::RemovedTestRule::check(),
                Rule::AnotherRemovedTestRule => test_rules::AnotherRemovedTestRule::check(),
                Rule::RedirectedToTestRule => test_rules::RedirectedToTestRule::check(),
                Rule::RedirectedFromTestRule => test_rules::RedirectedFromTestRule::check(),
                Rule::RedirectedFromPrefixTestRule => {
                    test_rules::RedirectedFromPrefixTestRule::check()
                }
                _ => unreachable!("All test rules must have an implementation"),
            };
            if let Some(diagnostic) = diagnostic {
                violations.push(diagnostic);
            }
        }
    }

    if (ignore_allow_comments.is_disabled() && !violations.is_empty())
        || rules.any_enabled(&[
            Rule::InvalidRuleCodeOrName,
            Rule::UnusedAllowComment,
            Rule::RedirectedAllowComment,
            Rule::DuplicatedAllowComment,
            Rule::DisabledAllowComment,
        ])
    {
        let ignored = check_allow_comments(&mut violations, &allow_comments, rules, file);
        if ignore_allow_comments.is_disabled() {
            for index in ignored.iter().rev() {
                violations.swap_remove(*index);
            }
        }
    }

    // Check violations for any remaining syntax errors. If any are found, discard violations
    // after it, as they may be false positives.
    if rules.enabled(Rule::SyntaxError) && root.has_error() {
        warn_user_once_by_message!(
            "Syntax errors detected in file: {}. Discarding subsequent violations from the AST.",
            path.to_string_lossy()
        );
        // Sort by byte-offset in the file
        violations.sort_by_key(|diagnostic| diagnostic.range.start());
        // Retain all violations up to the first syntax error, inclusive.
        // Text and path rules can be safely retained.
        let syntax_error_idx = violations
            .iter()
            .position(|diagnostic| diagnostic.kind.rule() == Rule::SyntaxError);
        if let Some(syntax_error_idx) = syntax_error_idx {
            violations = violations
                .into_iter()
                .enumerate()
                .filter_map(|(idx, diagnostic)| {
                    if idx <= syntax_error_idx || !diagnostic.kind.rule().is_ast_rule() {
                        Some(diagnostic)
                    } else {
                        None
                    }
                })
                .collect_vec();
        }
    }

    violations
}

const MAX_ITERATIONS: usize = 100;

pub type FixTable = FxHashMap<Rule, usize>;

pub struct FixerResult<'a> {
    /// The result returned by the linter, after applying any fixes.
    pub result: Vec<DiagnosticMessage>,
    /// The resulting source code, after applying any fixes.
    pub transformed: Cow<'a, SourceFile>,
    /// The number of fixes applied for each [`Rule`].
    pub fixed: FixTable,
}

#[allow(clippy::too_many_arguments)]
pub fn check_and_fix_file<'a>(
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    path: &Path,
    file: &'a SourceFile,
    settings: &Settings,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<FixerResult<'a>> {
    let mut transformed = Cow::Borrowed(file);

    // Track the number of fixed errors across iterations.
    let mut fixed = FxHashMap::default();

    // As an escape hatch, bail after 100 iterations.
    let mut iterations = 0;

    // Track whether the _initial_ source code is valid syntax.
    let mut is_valid_syntax = false;

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;

    // Continuously fix until the source code stabilizes.
    loop {
        let tree = parser
            .parse(transformed.source_text(), None)
            .context("Failed to parse")?;

        // Map row and column locations to byte slices (lazily).
        let locator = Locator::new(transformed.source_text());

        let violations = check_path(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &transformed,
            settings,
            &tree,
            ignore_allow_comments,
        );

        if iterations == 0 {
            is_valid_syntax = !tree.root_node().has_error();
            if !is_valid_syntax {
                warn_user_once_by_message!(
                    "Syntax errors detected in file: {}. No fixes will be applied.",
                    path.to_string_lossy()
                );
                return Err(anyhow!(
                    "File contains syntax errors, no fixes will be applied"
                ));
            }
        } else if is_valid_syntax && tree.root_node().has_error() {
            report_fix_syntax_error(path, transformed.source_text(), fixed.keys().copied());
            return Err(anyhow!("Fix introduced a syntax error"));
        }

        // Apply fix
        if let Some(FixResult {
            code: fixed_contents,
            fixes: applied,
            ..
        }) = fix_file(
            &violations,
            &locator,
            settings.check.unsafe_fixes,
            path.to_string_lossy().as_ref(),
        ) {
            if iterations < MAX_ITERATIONS {
                // Count the number of fixed errors
                for (rule, count) in applied {
                    *fixed.entry(rule).or_default() += count;
                }

                transformed = Cow::Owned(fixed_contents);

                iterations += 1;

                // Re-run the linter pass
                continue;
            }

            report_failed_to_converge_error(path, transformed.source_text(), &violations);
        };

        return Ok(FixerResult {
            result: violations
                .into_iter()
                .map(|v| DiagnosticMessage::from_ruff(&transformed, v))
                .collect_vec(),
            transformed,
            fixed,
        });
    }
}

fn collect_rule_codes(rules: impl IntoIterator<Item = Rule>) -> String {
    rules
        .into_iter()
        .map(|rule| rule.noqa_code().to_string())
        .sorted_unstable()
        .dedup()
        .join(", ")
}

#[allow(clippy::print_stderr)]
fn report_failed_to_converge_error(path: &Path, transformed: &str, diagnostics: &[Diagnostic]) {
    let codes = collect_rule_codes(diagnostics.iter().map(|diagnostic| diagnostic.kind.rule()));
    if cfg!(debug_assertions) {
        eprintln!(
            "{}{} Failed to converge after {} iterations in `{}` with rule codes {}:---\n{}\n---",
            "debug error".red().bold(),
            ":".bold(),
            MAX_ITERATIONS,
            fs::relativize_path(path),
            codes,
            transformed,
        );
    } else {
        eprintln!(
            r#"
{}{} Failed to converge after {} iterations.

This indicates a bug in fortitude. If you could open an issue at:

    https://github.com/PlasmaFAIR/fortitude/issues/new?title=%5BInfinite%20loop%5D

...quoting the contents of `{}`, the rule codes {}, along with the `fpm.toml` settings and executed command, we'd be very appreciative!
"#,
            "error".red().bold(),
            ":".bold(),
            MAX_ITERATIONS,
            fs::relativize_path(path),
            codes
        );
    }
}

#[allow(clippy::print_stderr)]
fn report_fix_syntax_error(path: &Path, transformed: &str, rules: impl IntoIterator<Item = Rule>) {
    // TODO: include syntax error
    let codes = collect_rule_codes(rules);
    if cfg!(debug_assertions) {
        eprintln!(
            "{}{} Fix introduced a syntax error in `{}` with rule codes {codes}: \n---\n{transformed}\n---",
            "error".red().bold(),
            ":".bold(),
            fs::relativize_path(path),
        );
    } else {
        eprintln!(
            r#"
{}{} Fix introduced a syntax error. Reverting all changes.

This indicates a bug in Fortitude. If you could open an issue at:

    https://github.com/PlasmaFAIR/fortitude/issues/new?title=%5BFix%20error%5D

...quoting the contents of `{}`, the rule codes {}, along with the `fortitude.toml`/`fpm.toml` settings and executed command, we'd be very appreciative!
"#,
            "error".red().bold(),
            ":".bold(),
            fs::relativize_path(path),
            codes,
        );
    }
}

pub fn rules_to_path_rules(rules: &RuleTable) -> Vec<PathRuleEnum> {
    rules
        .iter_enabled()
        .filter_map(|rule| TryFrom::try_from(rule).ok())
        .collect_vec()
}

pub fn rules_to_text_rules(rules: &RuleTable) -> Vec<TextRuleEnum> {
    rules
        .iter_enabled()
        .filter_map(|rule| TryFrom::try_from(rule).ok())
        .collect_vec()
}

/// Create a mapping of AST entrypoints to lists of the rules and codes that operate on them.
pub fn ast_entrypoint_map<'a>(rules: &RuleTable) -> BTreeMap<&'a str, Vec<AstRuleEnum>> {
    let ast_rules: Vec<AstRuleEnum> = rules
        .iter_enabled()
        .filter_map(|rule| TryFrom::try_from(rule).ok())
        .collect();

    let mut map: BTreeMap<&'a str, Vec<_>> = BTreeMap::new();
    for rule in ast_rules {
        for entrypoint in rule.entrypoints() {
            match map.get_mut(entrypoint) {
                Some(rule_vec) => {
                    rule_vec.push(rule);
                }
                None => {
                    map.insert(entrypoint, vec![rule]);
                }
            }
        }
    }
    map
}

/// Simplify making a `SourceFile` in tests
#[cfg(test)]
pub fn test_file(source: &str) -> SourceFile {
    use ruff_source_file::SourceFileBuilder;
    use textwrap::dedent;

    SourceFileBuilder::new("test.f90", dedent(source)).finish()
}
