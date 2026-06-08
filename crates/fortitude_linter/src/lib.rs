use line_filter::FilterSet;
use ruff_text_size::Ranged;
pub use rule_selector::RuleSelector;
use rule_table::RuleTable;
pub use settings::Settings;

pub mod allow_comments;
pub mod ast;
pub mod diagnostics;
pub mod fix;
pub mod fs;
pub mod line_filter;
pub mod line_width;
pub mod locator;
#[macro_use]
pub mod logging;
pub mod registry;
pub mod rule_redirects;
pub mod rule_selector;
pub mod rule_table;
pub mod rules;
pub mod settings;
pub mod source_kind;
pub mod stylist;
#[cfg(test)]
mod test;
pub mod text_helpers;
pub mod traits;
pub mod whitespace;

use allow_comments::{check_allow_comments, gather_allow_comments};
use ast::FortitudeNode;
use diagnostics::{Diagnostic, Diagnostics, FixMap, Violation};
use fix::{FixResult, fix_file};
use locator::Locator;
use rules::correctness::shadowed_variable::check_shadowed_variables;
use rules::correctness::split_escaped_quote::SplitEscapedQuote;
use rules::error::invalid_character::check_invalid_character;
use rules::error::syntax_error::SyntaxError;
use rules::style::file_extensions::NonStandardFileExtension;
use rules::style::inconsistent_dimension::check_inconsistent_dimension_rules;
use rules::style::keywords::check_keyword_reuse;
use rules::style::line_length::LineTooLong;
use rules::style::useless_return::check_superfluous_returns;
use rules::style::whitespace::{
    MissingNewlineAtEndOfFile, TrailingWhitespace, check_incorrect_indent,
};
#[cfg(any(feature = "test-rules", test))]
use rules::testing::test_rules::{self, TEST_RULES, TestRule};
use rules::{Rule, portability::invalid_tab::check_invalid_tab};
use settings::{CheckSettings, FixMode};
use stylist::Stylist;
use traits::TextRanged;

use anyhow::{Context, anyhow};
use ast::symbol_table::{BEGIN_SCOPE_NODES, END_SCOPE_NODES, SymbolTable, SymbolTables};
use colored::Colorize;
use itertools::Itertools;
use ruff_source_file::{SourceFile, SourceFileBuilder};
use rustc_hash::FxHashMap;
use source_kind::SourceKindDiff;
use std::borrow::Cow;
use std::fs::File;
use std::io::{self, Write};
use std::iter::once;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Rule trait
// ----------

/// Implemented by rules that analyse the abstract syntax tree.
pub trait AstRule {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>>;

    /// Return list of tree-sitter node types on which a rule should trigger.
    fn entrypoints() -> Vec<&'static str>;
}

pub struct CheckContext<'a> {
    file: SourceFile,
    rules: RuleTable,
    settings: &'a CheckSettings,
    stylist: &'a Stylist<'a>,
    symbols: SymbolTables<'a>,
}

impl<'a> CheckContext<'a> {
    pub fn new(
        path: &Path,
        contents: &str,
        settings: &'a CheckSettings,
        stylist: &'a Stylist,
    ) -> Self {
        let file = SourceFileBuilder::new(path.to_string_lossy(), contents).finish();

        // Ignore diagnostics based on per-file-ignores.
        let mut rules = settings.rules.clone();
        for ignore in crate::fs::ignores_from_path(path, &settings.per_file_ignores) {
            rules.disable(ignore);
        }
        let symbols = SymbolTables::default();

        Self {
            file,
            rules,
            settings,
            stylist,
            symbols,
        }
    }

    #[inline]
    pub const fn is_rule_enabled(&self, rule: Rule) -> bool {
        self.rules.enabled(rule)
    }

    #[inline]
    pub const fn any_rule_enabled(&self, rules: &[Rule]) -> bool {
        self.rules.any_enabled(rules)
    }

    /// Returns the source file.
    pub const fn source_file(&self) -> &SourceFile {
        &self.file
    }

    /// Returns the source code.
    #[inline]
    pub fn source_text(&self) -> &str {
        self.file.source_text()
    }

    /// The [`CheckSettings`] for the current analysis, including the enabled rules.
    pub const fn settings(&self) -> &'a CheckSettings {
        self.settings
    }

    /// The [`Stylist`] for the current file, which detects the current line ending, quote, and
    /// indentation style.
    pub const fn stylist(&self) -> &'a Stylist<'a> {
        self.stylist
    }

    pub const fn symbol_table(&'a self) -> &'a SymbolTables<'a> {
        &self.symbols
    }

    pub fn push_table(&mut self, table: SymbolTable<'a>) {
        self.symbols.push_table(table);
    }

    pub fn pop_table(&mut self) {
        self.symbols.pop_table();
    }

    #[must_use]
    pub fn create_diagnostic<T: Violation, R: TextRanged>(&self, kind: T, range: R) -> Diagnostic {
        Diagnostic::new(kind, range.textrange())
    }

    #[must_use]
    pub fn create_diagnostic_if_enabled<T: Violation, R: TextRanged>(
        &self,
        kind: T,
        range: R,
    ) -> Option<Diagnostic> {
        let rule = T::rule();
        if self.is_rule_enabled(rule) {
            Some(self.create_diagnostic(kind, range))
        } else {
            None
        }
    }
}

/// Parse a file, check it for issues, and return the report.
#[allow(clippy::too_many_arguments)]
pub fn check_file(
    path: &Path,
    file: &SourceFile,
    line_filter: &Option<FilterSet>,
    settings: &CheckSettings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Diagnostics> {
    let (mut messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(path, file, settings, ignore_allow_comments)
        {
            if !fixed.is_empty() {
                match fix_mode {
                    FixMode::Apply => {
                        let mut out_file = File::create(path)?;
                        out_file.write_all(transformed.source_text().as_bytes())?;
                    }
                    FixMode::Diff => {
                        write!(
                            &mut io::stdout().lock(),
                            "{}",
                            SourceKindDiff::new(file, &transformed, Some(path))
                        )?;
                    }
                    FixMode::Generate => {}
                }
            }

            (result, fixed)
        } else {
            // Failed to fix, so just lint the original source
            let result = check_only_file(path, file, settings, ignore_allow_comments)?;
            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(path, file, settings, ignore_allow_comments)?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    if let Some(line_filter) = line_filter {
        messages.retain(|message| line_filter.contains(message.start()));
    }

    Ok(Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    })
}

/// Parse a file, check it for issues, and return the report.
pub fn check_only_file(
    path: &Path,
    file: &SourceFile,
    settings: &CheckSettings,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> anyhow::Result<Vec<Diagnostic>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;
    let tree = parser
        .parse(file.source_text(), None)
        .context("Failed to parse")?;

    let violations = check_path(path, file, settings, &tree, ignore_allow_comments);

    Ok(violations
        .into_iter()
        .map(|v| v.with_file(file.clone()))
        .collect_vec())
}

/// Check an already parsed file. This actually does all the checking,
/// `check_only_file`/`check_and_fix_file` wrap this
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_path(
    path: &Path,
    file: &SourceFile,
    settings: &CheckSettings,
    tree: &Tree,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Vec<Diagnostic> {
    let mut violations = Vec::new();
    let mut allow_comments = Vec::new();

    // Detect the current code style (lazily)
    let stylist = Stylist::from_ast(&tree.root_node(), file);

    let mut context = CheckContext::new(path, file.source_text(), settings, &stylist);

    // Check file paths directly
    if context.is_rule_enabled(Rule::NonStandardFileExtension)
        && let Some(violation) = NonStandardFileExtension::check(&context)
    {
        violations.push(violation);
    }

    // Perform plain text analysis
    if context.is_rule_enabled(Rule::SplitEscapedQuote) {
        violations.extend(SplitEscapedQuote::check(&context));
    }
    if context.is_rule_enabled(Rule::TrailingWhitespace) {
        violations.extend(TrailingWhitespace::check(&context));
    }
    if context.is_rule_enabled(Rule::MissingNewlineAtEndOfFile)
        && let Some(violation) = MissingNewlineAtEndOfFile::check(&context)
    {
        violations.push(violation);
    }

    // Perform AST analysis
    let root = tree.root_node();
    for node in once(root).chain(root.descendants()) {
        if context.is_rule_enabled(Rule::SyntaxError) && node.is_missing() {
            violations.push(context.create_diagnostic(SyntaxError {}, node));
        }

        if node.is_named() && BEGIN_SCOPE_NODES.contains(&node.kind()) {
            let new_table = SymbolTable::new(&node, file.source_text());

            // Check for keyword reuse in this scope
            if context.rules.enabled(Rule::KeywordReuse) {
                violations.extend(check_keyword_reuse(&context, &new_table));
            }

            // Check for shadowed variables in this scope
            if context.rules.enabled(Rule::ShadowedVariable) {
                violations.extend(check_shadowed_variables(&context, &new_table))
            }

            // Run rules over variable declarations without needing to reparse
            // them into types
            if context.any_rule_enabled(&[
                Rule::InconsistentArrayDeclaration,
                Rule::MixedScalarArrayDeclaration,
                Rule::BadArrayDeclaration,
            ]) {
                for decl_line in new_table.iter_decl_lines() {
                    violations.extend(check_inconsistent_dimension_rules(&context, decl_line))
                }
            }

            context.push_table(new_table);
        }

        if context.any_rule_enabled(&[
            Rule::SuperfluousElseReturn,
            Rule::SuperfluousElseCycle,
            Rule::SuperfluousElseExit,
            Rule::SuperfluousElseStop,
        ]) && matches!(node.kind(), "keyword_statement" | "stop_statement")
            && let Some(violation) = check_superfluous_returns(&context, &node)
        {
            violations.push(violation);
        }

        if let Some(rules) = context.rules.ast_entrypoints().get(node.kind()) {
            for rule in rules {
                if let Some(violation) = rule.check(&context, &node) {
                    violations.extend(violation);
                }
            }
        }

        if let Some(allow_rules) = gather_allow_comments(&node, file) {
            allow_comments.push(allow_rules);
        };

        if END_SCOPE_NODES.contains(&node.kind()) {
            context.pop_table();
        }
    }

    // ignore line length in comments requires AST
    if context.is_rule_enabled(Rule::LineTooLong) {
        violations.extend(LineTooLong::check(&context, &root));
    }

    if context.is_rule_enabled(Rule::InvalidTab) {
        violations.append(&mut check_invalid_tab(&context, &root));
    }

    if context.is_rule_enabled(Rule::IncorrectIndent) {
        violations.append(&mut &mut check_incorrect_indent(&context, &root));
    }

    if context.is_rule_enabled(Rule::InvalidCharacter) {
        violations.append(&mut check_invalid_character(&context, &root));
    }

    // Raise violations for internal test rules
    #[cfg(any(feature = "test-rules", test))]
    {
        for test_rule in TEST_RULES {
            if !context.is_rule_enabled(*test_rule) {
                continue;
            }
            let diagnostic = match test_rule {
                Rule::StableTestRule => test_rules::StableTestRule::check(&context),
                Rule::StableTestRuleSafeFix => test_rules::StableTestRuleSafeFix::check(&context),
                Rule::StableTestRuleUnsafeFix => {
                    test_rules::StableTestRuleUnsafeFix::check(&context)
                }
                Rule::StableTestRuleDisplayOnlyFix => {
                    test_rules::StableTestRuleDisplayOnlyFix::check(&context)
                }
                Rule::PreviewTestRule => test_rules::PreviewTestRule::check(&context),
                Rule::DeprecatedTestRule => test_rules::DeprecatedTestRule::check(&context),
                Rule::AnotherDeprecatedTestRule => {
                    test_rules::AnotherDeprecatedTestRule::check(&context)
                }
                Rule::RemovedTestRule => test_rules::RemovedTestRule::check(&context),
                Rule::AnotherRemovedTestRule => test_rules::AnotherRemovedTestRule::check(&context),
                Rule::RedirectedToTestRule => test_rules::RedirectedToTestRule::check(&context),
                Rule::RedirectedFromTestRule => test_rules::RedirectedFromTestRule::check(&context),
                Rule::RedirectedFromPrefixTestRule => {
                    test_rules::RedirectedFromPrefixTestRule::check(&context)
                }
                _ => unreachable!("All test rules must have an implementation"),
            };
            if let Some(diagnostic) = diagnostic {
                violations.push(diagnostic);
            }
        }
    }

    if (ignore_allow_comments.is_disabled() && !violations.is_empty())
        || context.any_rule_enabled(&[
            Rule::InvalidRuleCodeOrName,
            Rule::UnusedAllowComment,
            Rule::RedirectedAllowComment,
            Rule::DuplicatedAllowComment,
            Rule::DisabledAllowComment,
        ])
    {
        let ignored = check_allow_comments(&mut violations, &allow_comments, &context);
        if ignore_allow_comments.is_disabled() {
            for index in ignored.iter().rev() {
                violations.swap_remove(*index);
            }
        }
    }

    // Handle syntax errors
    if root.has_error() {
        // If syntax error violations are present, we can (probably) trust AST
        // violations up to the first syntax error. If we aren't tracking syntax
        // errors, we report everything but warn that the results are unreliable.
        // In either case, fixes should be considered too risky to apply.
        if context.is_rule_enabled(Rule::SyntaxError) {
            // Check violations for any remaining syntax errors. If any are found, discard violations
            // after it, as they may be false positives.
            warn_user_once_by_message!(
                "Syntax errors detected in file: {}. Discarding subsequent violations from the AST and all fixes.",
                path.to_string_lossy()
            );
            // Sort by byte-offset in the file
            violations.sort_by_key(|diagnostic| diagnostic.range().start());
            // Retain all violations up to the first syntax error, inclusive.
            // Text and path rules can be safely retained.
            let syntax_error_idx = violations
                .iter()
                .position(|diagnostic| diagnostic.rule() == Rule::SyntaxError);
            if let Some(syntax_error_idx) = syntax_error_idx {
                violations = violations
                    .into_iter()
                    .enumerate()
                    .filter_map(|(idx, diagnostic)| {
                        if idx <= syntax_error_idx || !diagnostic.rule().is_ast_rule() {
                            Some(diagnostic)
                        } else {
                            None
                        }
                    })
                    .collect_vec();
            }
        } else {
            // If syntax errors are present but the rule is disabled, just warn
            // that false positives may be present.
            warn_user_once_by_message!(
                "Syntax errors detected in file: {}. Discarding all fixes. Some violations from the AST may be unreliable.",
                path.to_string_lossy()
            );
        }
        // Disable all fixes
        for diagnostic in &mut violations {
            diagnostic.drop_fix();
        }
    }

    // Disable any fixes for unfixable rules
    for diagnostic in &mut violations {
        let rule = diagnostic.rule();
        if diagnostic.fixable() && !context.rules.should_fix(rule) {
            diagnostic.drop_fix();
        }
    }

    violations
}

const MAX_ITERATIONS: usize = 100;

pub type FixTable = FxHashMap<Rule, usize>;

pub struct FixerResult<'a> {
    /// The result returned by the linter, after applying any fixes.
    pub result: Vec<Diagnostic>,
    /// The resulting source code, after applying any fixes.
    pub transformed: Cow<'a, SourceFile>,
    /// The number of fixes applied for each [`Rule`].
    pub fixed: FixTable,
}

#[allow(clippy::too_many_arguments)]
pub fn check_and_fix_file<'a>(
    path: &Path,
    file: &'a SourceFile,
    settings: &CheckSettings,
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

        let violations = check_path(path, &transformed, settings, &tree, ignore_allow_comments);

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
            settings.unsafe_fixes,
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
                .map(|v| v.with_file(transformed.clone().into_owned()))
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
    let codes = collect_rule_codes(diagnostics.iter().map(|diagnostic| diagnostic.rule()));
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

...quoting the contents of `{}`, the rule codes {}, along with any settings files and the executed command, we'd be very appreciative!
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

...quoting the contents of `{}`, the rule codes {}, along with the any settings files and the executed command, we'd be very appreciative!
"#,
            "error".red().bold(),
            ":".bold(),
            fs::relativize_path(path),
            codes,
        );
    }
}

/// Simplify making a `SourceFile` in tests
#[cfg(test)]
pub fn test_file(source: &str) -> SourceFile {
    use ruff_source_file::SourceFileBuilder;
    use textwrap::dedent;

    SourceFileBuilder::new("test.f90", dedent(source)).finish()
}
