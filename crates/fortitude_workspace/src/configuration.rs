use crate::options::{
    ExitUnlabelledLoopOptions, InconsistentDimensionOptions, InvalidTabOptions,
    KeywordWhitespaceOptions, Options, PortabilityOptions, StringOptions,
};
use fortitude_linter::fs::{
    EXCLUDE_BUILTINS, FORTRAN_EXTS, FilePattern, FilePatternSet, GlobPath, INCLUDE,
};
use fortitude_linter::registry::RuleNamespace;
use fortitude_linter::rule_redirects::get_redirect;
use fortitude_linter::rule_selector::{
    CompiledPerFileIgnoreList, PerFileIgnore, PreviewOptions, RuleSelector, Specificity,
};
use fortitude_linter::rule_table::RuleTable;
use fortitude_linter::rules::Rule;
use fortitude_linter::settings::{
    CheckSettings, DEFAULT_SELECTORS, ExcludeMode, FileResolverSettings, FortranStandard,
    GitignoreMode, OutputFormat, PreviewMode, ProgressBar, Settings, UnsafeFixes,
};
use fortitude_linter::{ast_entrypoint_map, fs, warn_user_once_by_id, warn_user_once_by_message};

use anyhow::{Context, Result, anyhow};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;

// These are just helper structs to let us quickly work out if there's
// a fortitude section in an fpm.toml file
#[derive(Debug, PartialEq, Eq, Default, Deserialize)]
struct Fpm {
    extra: Option<Extra>,
}

#[derive(Debug, PartialEq, Eq, Default, Deserialize)]
struct Extra {
    fortitude: Option<Options>,
}

// Adapted from ruff
fn parse_fpm_toml<P: AsRef<Path>>(path: P) -> Result<Fpm> {
    let contents = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.as_ref().display()))
}

fn parse_fortitude_toml<P: AsRef<Path>>(path: P) -> Result<Options> {
    let contents = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.as_ref().display()))
}

pub fn fortitude_enabled<P: AsRef<Path>>(path: P) -> Result<bool> {
    let fpm = parse_fpm_toml(path)?;
    Ok(fpm.extra.and_then(|extra| extra.fortitude).is_some())
}

/// Return the path to the `fpm.toml` or `fortitude.toml` file in a given
/// directory. Adapted from ruff
pub fn settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    // Check for `.fortitude.toml`.
    let fortitude_toml = path.as_ref().join(".fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fortitude.toml`.
    let fortitude_toml = path.as_ref().join("fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fpm.toml`.
    let fpm_toml = path.as_ref().join("fpm.toml");
    if fpm_toml.is_file() && fortitude_enabled(&fpm_toml)? {
        return Ok(Some(fpm_toml));
    }

    Ok(None)
}

/// Find the path to the `fpm.toml` or `fortitude.toml` file, if such a file
/// exists. Adapted from ruff
pub fn find_settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    for directory in path.as_ref().ancestors() {
        if let Some(settings) = settings_toml(directory)? {
            return Ok(Some(settings));
        }
    }
    Ok(None)
}

/// Find the path to the user-specific `fpm.toml` or `fortitude.toml`, if it
/// exists.
#[cfg(not(target_arch = "wasm32"))]
pub fn find_user_settings_toml() -> Option<PathBuf> {
    use etcetera::BaseStrategy;

    let strategy = etcetera::base_strategy::choose_base_strategy().ok()?;
    let config_dir = strategy.config_dir().join("fortitude");

    // Search for a user-specific `.fortitude.toml`, then a `fortitude.toml`, then a `fpm.toml`.
    for filename in [".fortitude.toml", "fortitude.toml", "fpm.toml"] {
        let path = config_dir.join(filename);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

/// Find the path to the project root, which contains the `fpm.toml` or `fortitude.toml` file.
/// If no such file exists, return the current working directory.
pub fn project_root<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    find_settings_toml(&path)?.map_or(Ok(fs::normalize_path(&path)), |settings| {
        fs::normalize_path(settings)
            .parent()
            .context("Settings file has no parent")
            .map(PathBuf::from)
    })
}

/// Read either the "extra.fortitude" table from "fpm.toml", or the
/// whole "fortitude.toml" file
pub fn load_options<P: AsRef<Path>>(path: P) -> Result<Options> {
    if path.as_ref().ends_with("fpm.toml") {
        let config = parse_fpm_toml(&path)?;
        // Unwrap should be ok here because we've already checked this
        // file has these tables
        Ok(config.extra.unwrap().fortitude.unwrap())
    } else {
        parse_fortitude_toml(&path)
    }
}

/// Resolve `--foo` and `--no-foo` arguments
pub fn resolve_bool_arg(yes: Option<bool>, no: Option<bool>) -> Option<bool> {
    let yes = yes.unwrap_or_default();
    let no = no.unwrap_or_default();
    match (yes, no) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (..) => unreachable!("Clap should make this impossible"),
    }
}

/// Convert the deprecated `files` and `file_extensions` settings to the new `include`
pub fn convert_file_and_extensions_to_include(
    paths: &Option<Vec<PathBuf>>,
    extensions: &Option<Vec<String>>,
) -> Option<Vec<String>> {
    if paths.is_none() && extensions.is_none() {
        return None;
    }

    let extensions = extensions
        .clone()
        .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect())
        .join(",");

    match paths {
        Some(paths) => {
            let (dirs, files): (Vec<_>, Vec<_>) = paths
                .iter()
                .map(fs::normalize_path)
                .unique()
                .partition(|p| p.is_dir());

            let mut include = dirs
                .iter()
                .map(|path| path.to_string_lossy())
                .map(|path| format!("{path}/*.{{{extensions}}}"))
                .collect_vec();

            include.extend(files.iter().map(|path| path.to_string_lossy().into_owned()));
            Some(include)
        }
        None => Some(vec![format!("*.{{{extensions}}}")]),
    }
}

// This is our "known good" intermediate settings struct after we've
// read the config file, but before we've overridden it from the CLI
#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub ignore: Vec<RuleSelector>,
    pub select: Option<Vec<RuleSelector>>,
    pub extend_select: Vec<RuleSelector>,
    pub per_file_ignores: Option<Vec<PerFileIgnore>>,
    pub extend_per_file_ignores: Vec<PerFileIgnore>,
    pub line_length: Option<usize>,
    pub fix: Option<bool>,
    pub fix_only: Option<bool>,
    pub show_fixes: Option<bool>,
    pub unsafe_fixes: Option<UnsafeFixes>,
    pub output_format: Option<OutputFormat>,
    pub target_std: Option<FortranStandard>,
    pub progress_bar: Option<ProgressBar>,
    pub preview: Option<PreviewMode>,

    // File resolver options
    pub include: Option<Vec<FilePattern>>,
    pub exclude: Option<Vec<FilePattern>>,
    pub extend_exclude: Vec<FilePattern>,
    pub force_exclude: Option<bool>,
    pub respect_gitignore: Option<bool>,

    // Individual rules
    pub exit_unlabelled_loops: Option<ExitUnlabelledLoopOptions>,
    pub keyword_whitespace: Option<KeywordWhitespaceOptions>,
    pub strings: Option<StringOptions>,
    pub portability: Option<PortabilityOptions>,
    pub invalid_tab: Option<InvalidTabOptions>,
    pub inconsistent_dimension: Option<InconsistentDimensionOptions>,
}

impl Configuration {
    /// Convert from config file options struct into our "known good" struct
    #[allow(deprecated)]
    pub fn from_options(options: Options, project_root: &Path) -> Self {
        let check = options.check.unwrap_or_default();

        let include = convert_file_and_extensions_to_include(&check.files, &check.file_extensions);

        Self {
            ignore: check.ignore.into_iter().flatten().collect(),
            select: check.select,
            extend_select: check.extend_select.unwrap_or_default(),
            per_file_ignores: check.per_file_ignores.map(|per_file_ignores| {
                per_file_ignores
                    .into_iter()
                    .map(|(pattern, prefixes)| {
                        PerFileIgnore::new(pattern, &prefixes, Some(project_root))
                    })
                    .collect()
            }),
            extend_per_file_ignores: vec![],
            line_length: check.line_length,
            fix: check.fix,
            fix_only: check.fix_only,
            show_fixes: check.show_fixes,
            unsafe_fixes: check.unsafe_fixes.map(UnsafeFixes::from),
            output_format: check.output_format,
            target_std: check.target_std,
            progress_bar: check.progress_bar,
            preview: check.preview.map(PreviewMode::from),
            include: options.include.or(include).map(|paths| {
                paths
                    .into_iter()
                    .map(|pattern| {
                        let absolute = GlobPath::normalize(&pattern, project_root);
                        FilePattern::User(pattern, absolute)
                    })
                    .collect()
            }),
            exclude: check.exclude.map(|paths| {
                paths
                    .into_iter()
                    .map(|pattern| {
                        let absolute = GlobPath::normalize(&pattern, project_root);
                        FilePattern::User(pattern, absolute)
                    })
                    .collect()
            }),
            extend_exclude: check
                .extend_exclude
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|pattern| {
                            let absolute = GlobPath::normalize(&pattern, project_root);
                            FilePattern::User(pattern, absolute)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            force_exclude: check.force_exclude,
            respect_gitignore: check.respect_gitignore,

            // Individual rules
            exit_unlabelled_loops: check.exit_unlabelled_loops,
            keyword_whitespace: check.keyword_whitespace,
            strings: check.strings,
            portability: check.portability,
            invalid_tab: check.invalid_tab,
            inconsistent_dimension: check.inconsistent_dimensions,
        }
    }

    pub fn into_settings(self, project_root: &Path) -> Result<Settings> {
        let preview = self.preview.unwrap_or_default();

        let rule_selection = RuleSelection {
            select: self.select,
            ignore: self.ignore,
            extend_select: self.extend_select,
            fixable: None,
            unfixable: vec![],
            extend_fixable: vec![],
        };
        let rules = to_rule_table(rule_selection, &preview)?;
        let ast_entrypoints = ast_entrypoint_map(&rules);

        let mut progress_bar = self.progress_bar.unwrap_or_default();
        // Override progress bar settings if not using colour terminal
        if progress_bar == ProgressBar::Fancy
            && !colored::control::SHOULD_COLORIZE.should_colorize()
        {
            progress_bar = ProgressBar::Ascii;
        }

        Ok(Settings {
            check: CheckSettings {
                project_root: project_root.to_path_buf(),
                rules,
                ast_entrypoints,
                fix: self.fix.unwrap_or_default(),
                fix_only: self.fix_only.unwrap_or_default(),
                line_length: self
                    .line_length
                    .unwrap_or(Settings::default().check.line_length),
                unsafe_fixes: self.unsafe_fixes.unwrap_or_default(),
                preview,
                target_std: self.target_std.unwrap_or_default(),
                progress_bar,
                output_format: self.output_format.unwrap_or_default(),
                show_fixes: self.show_fixes.unwrap_or_default(),
                per_file_ignores: CompiledPerFileIgnoreList::resolve(
                    self.per_file_ignores
                        .unwrap_or_default()
                        .into_iter()
                        .chain(self.extend_per_file_ignores)
                        .collect(),
                )?,

                // Individual rules
                exit_unlabelled_loops: self
                    .exit_unlabelled_loops
                    .map(ExitUnlabelledLoopOptions::into_settings)
                    .unwrap_or_default(),
                keyword_whitespace: self
                    .keyword_whitespace
                    .map(KeywordWhitespaceOptions::into_settings)
                    .unwrap_or_default(),
                strings: self
                    .strings
                    .map(StringOptions::into_settings)
                    .unwrap_or_default(),
                portability: self
                    .portability
                    .map(PortabilityOptions::into_settings)
                    .unwrap_or_default(),
                invalid_tab: self
                    .invalid_tab
                    .map(InvalidTabOptions::into_settings)
                    .unwrap_or_default(),
                inconsistent_dimension: self
                    .inconsistent_dimension
                    .map(InconsistentDimensionOptions::into_settings)
                    .unwrap_or_default(),
            },
            file_resolver: FileResolverSettings {
                project_root: project_root.to_path_buf(),
                exclude: FilePatternSet::try_from_iter(
                    self.exclude.unwrap_or_else(|| EXCLUDE_BUILTINS.to_vec()),
                )?,
                extend_exclude: FilePatternSet::try_from_iter(self.extend_exclude)?,
                include: FilePatternSet::try_from_iter(
                    self.include.unwrap_or_else(|| INCLUDE.to_vec()),
                )?,
                respect_gitignore: self
                    .respect_gitignore
                    .map(GitignoreMode::from)
                    .unwrap_or_default()
                    .is_respect_gitignore(),
                force_exclude: self
                    .force_exclude
                    .map(ExcludeMode::from)
                    .unwrap_or_default()
                    .is_force(),
            },
        })
    }

    #[must_use]
    pub fn combine(self, config: Self) -> Self {
        Self {
            include: self.include.or(config.include),
            ignore: self.ignore.into_iter().chain(config.ignore).collect(),
            select: self.select.or(config.select),
            extend_select: self
                .extend_select
                .into_iter()
                .chain(config.extend_select)
                .collect(),
            per_file_ignores: self.per_file_ignores.or(config.per_file_ignores),
            extend_per_file_ignores: self
                .extend_per_file_ignores
                .into_iter()
                .chain(config.extend_per_file_ignores)
                .collect(),
            line_length: self.line_length.or(config.line_length),
            fix: self.fix.or(config.fix),
            fix_only: self.fix_only.or(config.fix_only),
            show_fixes: self.show_fixes.or(config.show_fixes),
            unsafe_fixes: self.unsafe_fixes.or(config.unsafe_fixes),
            output_format: self.output_format.or(config.output_format),
            progress_bar: self.progress_bar.or(config.progress_bar),
            preview: self.preview.or(config.preview),
            exclude: self.exclude.or(config.exclude),
            extend_exclude: self
                .extend_exclude
                .into_iter()
                .chain(config.extend_exclude)
                .collect(),
            force_exclude: self.force_exclude.or(config.force_exclude),
            respect_gitignore: self.respect_gitignore.or(config.respect_gitignore),
            exit_unlabelled_loops: self.exit_unlabelled_loops.or(config.exit_unlabelled_loops),
            keyword_whitespace: self.keyword_whitespace.or(config.keyword_whitespace),
            strings: self.strings.or(config.strings),
            portability: self.portability.or(config.portability),
            invalid_tab: self.invalid_tab.or(config.invalid_tab),
            target_std: self.target_std.or(config.target_std),
            inconsistent_dimension: self
                .inconsistent_dimension
                .or(config.inconsistent_dimension),
        }
    }
}

/// Applies a transformation to a [`Configuration`].
///
/// Used to override options with the values provided by the CLI.
pub trait ConfigurationTransformer {
    fn transform(&self, config: Configuration) -> Configuration;
}

/// Get the list of active rules for this session.
pub fn to_rule_table(args: RuleSelection, preview: &PreviewMode) -> anyhow::Result<RuleTable> {
    let preview = PreviewOptions {
        mode: *preview,
        require_explicit: false,
    };

    // Store selectors for displaying warnings
    let mut redirects = FxHashMap::default();
    let mut deprecated_selectors = FxHashSet::default();
    let mut removed_selectors = FxHashSet::default();
    let mut removed_ignored_rules = FxHashSet::default();
    let mut ignored_preview_selectors = FxHashSet::default();

    // The select_set keeps track of which rules have been selected.
    let mut select_set: BTreeSet<Rule> = if args.select.is_none() {
        DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect()
    } else {
        BTreeSet::default()
    };

    for spec in Specificity::iter() {
        // Iterate over rule selectors in order of specificity.
        for selector in args
            .select
            .iter()
            .flatten()
            .chain(args.extend_select.iter())
            .filter(|s| s.specificity() == spec)
        {
            for rule in selector.rules(&preview) {
                select_set.insert(rule);
            }
        }

        for selector in args.ignore.iter().filter(|s| s.specificity() == spec) {
            for rule in selector.rules(&preview) {
                select_set.remove(&rule);
            }
        }

        // Check for selections that require a warning
        for (kind, selector) in args.selectors_by_kind() {
            // Some of these checks are only for `Kind::Enable` which means only `--select` will warn
            // and use with, e.g., `--ignore` or `--fixable` is okay

            // Unstable rules
            if preview.mode.is_disabled() && kind.is_enable() {
                // Check if the selector is empty because preview mode is disabled
                if selector.rules(&preview).next().is_none()
                    && selector
                        .rules(&PreviewOptions {
                            mode: PreviewMode::Enabled,
                            require_explicit: preview.require_explicit,
                        })
                        .next()
                        .is_some()
                {
                    ignored_preview_selectors.insert(selector);
                }
            }

            // Deprecated rules
            if kind.is_enable()
                && selector.is_exact()
                && selector.all_rules().all(|rule| rule.is_deprecated())
            {
                deprecated_selectors.insert(selector.clone());
            }

            // Removed rules
            if selector.is_exact() && selector.all_rules().all(|rule| rule.is_removed()) {
                if kind.is_disable() {
                    removed_ignored_rules.insert(selector);
                } else {
                    removed_selectors.insert(selector);
                }
            }

            // Redirected rules
            if let RuleSelector::Prefix {
                prefix,
                redirected_from: Some(redirect_from),
            }
            | RuleSelector::Rule {
                prefix,
                redirected_from: Some(redirect_from),
            } = selector
            {
                redirects.insert(redirect_from, prefix);
            }
            if let RuleSelector::DeprecatedCategory {
                rules,
                redirected_to,
                redirected_from,
            } = selector
            {
                warn_user_once_by_message!(
                    "The selector `{redirected_from}` refers to a deprecated rule category."
                );
                for (from, to) in rules.iter().zip(redirected_to.iter()) {
                    redirects.insert(from, to);
                }
            }
        }
    }

    let removed_selectors = removed_selectors.iter().sorted().collect::<Vec<_>>();
    match removed_selectors.as_slice() {
        [] => (),
        [selection] => {
            let (prefix, code) = selection.prefix_and_code();
            return Err(anyhow!(
                "Rule `{prefix}{code}` was removed and cannot be selected."
            ));
        }
        [..] => {
            let mut message =
                "The following rules have been removed and cannot be selected:".to_string();
            for selection in removed_selectors {
                let (prefix, code) = selection.prefix_and_code();
                message.push_str(format!("\n    - {prefix}{code}").as_str());
            }
            message.push('\n');
            return Err(anyhow!(message));
        }
    }

    if !removed_ignored_rules.is_empty() {
        let mut rules = String::new();
        for selection in removed_ignored_rules.iter().sorted() {
            let (prefix, code) = selection.prefix_and_code();
            rules.push_str(format!("\n    - {prefix}{code}").as_str());
        }
        rules.push('\n');
        warn_user_once_by_message!(
            "The following rules have been removed and ignoring them has no effect:{rules}"
        );
    }

    for (from, target) in redirects.iter().sorted_by_key(|item| item.0) {
        // Some redirects are from one long-code to another, but the target will
        // be converted to a short code as part of the redirection. To get around
        // this, we need to check the redirect map from scratch.
        let target = match get_redirect(from) {
            Some(target) => target.1.to_string(),
            None => format!(
                "{}{}",
                target.category().common_prefix(),
                target.short_code()
            ),
        };
        warn_user_once_by_id!(from, "`{from}` has been remapped to `{target}`.");
    }

    if preview.mode.is_disabled() {
        for selection in deprecated_selectors.iter().sorted() {
            let (prefix, code) = selection.prefix_and_code();
            warn_user_once_by_message!(
                "Rule `{prefix}{code}` is deprecated and will be removed in a future release."
            );
        }
    } else {
        let deprecated_selectors = deprecated_selectors.iter().sorted().collect::<Vec<_>>();
        match deprecated_selectors.as_slice() {
            [] => (),
            [selection] => {
                let (prefix, code) = selection.prefix_and_code();
                return Err(anyhow!(
                    "Selection of deprecated rule `{prefix}{code}` is not allowed when preview is enabled."
                ));
            }
            [..] => {
                let mut message = "Selection of deprecated rules is not allowed when preview is enabled. Remove selection of:".to_string();
                for selection in deprecated_selectors {
                    let (prefix, code) = selection.prefix_and_code();
                    message.push_str(format!("\n\t- {prefix}{code}").as_str());
                }
                message.push('\n');
                return Err(anyhow!(message));
            }
        }
    }

    for selection in ignored_preview_selectors.iter().sorted() {
        let (prefix, code) = selection.prefix_and_code();
        warn_user_once_by_message!(
            "Selection `{prefix}{code}` has no effect because preview is not enabled.",
        );
    }

    let mut rules = RuleTable::empty();

    for rule in select_set {
        let should_fix = true;
        rules.enable(rule, should_fix);
    }

    Ok(rules)
}

// Taken from Ruff
#[derive(Clone, Debug, Default)]
pub struct RuleSelection {
    pub select: Option<Vec<RuleSelector>>,
    pub ignore: Vec<RuleSelector>,
    pub extend_select: Vec<RuleSelector>,
    pub fixable: Option<Vec<RuleSelector>>,
    pub unfixable: Vec<RuleSelector>,
    pub extend_fixable: Vec<RuleSelector>,
}

#[derive(Debug, Eq, PartialEq, is_macro::Is)]
pub enum RuleSelectorKind {
    /// Enables the selected rules
    Enable,
    /// Disables the selected rules
    Disable,
    /// Modifies the behavior of selected rules
    Modify,
}

impl RuleSelection {
    pub fn selectors_by_kind(&self) -> impl Iterator<Item = (RuleSelectorKind, &RuleSelector)> {
        self.select
            .iter()
            .flatten()
            .map(|selector| (RuleSelectorKind::Enable, selector))
            .chain(
                self.fixable
                    .iter()
                    .flatten()
                    .map(|selector| (RuleSelectorKind::Modify, selector)),
            )
            .chain(
                self.ignore
                    .iter()
                    .map(|selector| (RuleSelectorKind::Disable, selector)),
            )
            .chain(
                self.extend_select
                    .iter()
                    .map(|selector| (RuleSelectorKind::Enable, selector)),
            )
            .chain(
                self.unfixable
                    .iter()
                    .map(|selector| (RuleSelectorKind::Modify, selector)),
            )
            .chain(
                self.extend_fixable
                    .iter()
                    .map(|selector| (RuleSelectorKind::Modify, selector)),
            )
    }
}

// This is required by the `options_group` macro, but we don't make
// use of it anywhere yet
#[allow(dead_code)]
pub(crate) trait CombinePluginOptions {
    #[must_use]
    fn combine(self, other: Self) -> Self;
}

impl<T: CombinePluginOptions> CombinePluginOptions for Option<T> {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Some(base), Some(other)) => Some(base.combine(other)),
            (Some(base), None) => Some(base),
            (None, Some(other)) => Some(other),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, create_dir};
    use std::str::FromStr;

    use fortitude_linter::{
        registry::RuleSet, rule_selector::RuleSelector, settings::DEFAULT_SELECTORS,
    };

    use crate::options::CheckOptions;

    use super::*;

    fn resolve_rules(args: RuleSelection, preview: &PreviewMode) -> Result<RuleSet> {
        Ok(to_rule_table(args, preview)?.iter_enabled().collect())
    }

    #[test]
    fn empty_select() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: None,
            extend_select: vec![],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = resolve_rules(args, &preview_mode)?;
        let preview = PreviewOptions::default();

        let all_rules: Vec<Rule> = DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect();

        let all_rules = RuleSet::from_rules(&all_rules);

        assert_eq!(rules, all_rules);

        Ok(())
    }

    #[test]
    fn empty_select_with_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: None,
            extend_select: vec![],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::Enabled;
        let rules = resolve_rules(args, &preview_mode)?;
        let preview = PreviewOptions {
            mode: preview_mode,
            require_explicit: false,
        };

        let all_rules: Vec<Rule> = DEFAULT_SELECTORS
            .iter()
            .flat_map(|selector| selector.rules(&preview))
            .collect();

        let all_rules = RuleSet::from_rules(&all_rules);

        assert_eq!(rules, all_rules);

        Ok(())
    }

    #[test]
    fn select_one_rule() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E000")?]),
            extend_select: vec![],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = resolve_rules(args, &preview_mode)?;
        let one_rules = RuleSet::from_rules(&[Rule::IoError]);

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn select_one_preview_rule_without_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("FORT9911")?]),
            extend_select: vec![],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = resolve_rules(args, &preview_mode)?;
        let one_rules = RuleSet::empty();

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn select_one_preview_rule_with_preview() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("FORT9911")?]),
            extend_select: vec![],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::Enabled;
        let rules = resolve_rules(args, &preview_mode)?;
        let one_rules = RuleSet::from_rule(Rule::PreviewTestRule);

        assert_eq!(rules, one_rules);

        Ok(())
    }

    #[test]
    fn extend_select() -> anyhow::Result<()> {
        let args = RuleSelection {
            ignore: vec![],
            select: Some(vec![RuleSelector::from_str("E000")?]),
            extend_select: vec![RuleSelector::from_str("E001")?],
            fixable: None,
            extend_fixable: vec![],
            unfixable: vec![],
        };

        let preview_mode = PreviewMode::default();
        let rules = resolve_rules(args, &preview_mode)?;
        let one_rules = RuleSet::from_rules(&[Rule::IoError, Rule::SyntaxError]);

        assert_eq!(rules, one_rules);

        Ok(())
    }

    use std::fs;

    use anyhow::{Context, Result};
    use tempfile::TempDir;
    use textwrap::dedent;

    #[test]
    fn find_and_check_fpm_toml() -> Result<()> {
        let tempdir = TempDir::new()?;
        let fpm_toml = tempdir.path().join("fpm.toml");
        fs::write(
            fpm_toml,
            dedent(
                r#"
                some-stuff = 1
                other-things = "hello"

                [extra.fortitude.check]
                ignore = ["T001"]
                "#,
            ),
        )?;

        let fpm = find_settings_toml(tempdir.path())?.context("Failed to find fpm.toml")?;
        let enabled = fortitude_enabled(fpm)?;
        assert!(enabled);

        Ok(())
    }

    #[test]
    fn convert_deprecated_file_and_extension() -> Result<()> {
        // Initialize the filesystem:
        //   root
        //   ├── file1.f90
        //   ├── dir1.f90
        //   │   └── file2.f90
        //   └── dir2.f90
        let tmp_dir = TempDir::new()?;
        let root = tmp_dir.path();
        let file1 = root.join("file1.f90");
        let dir1 = root.join("dir1.f90");
        let file2 = dir1.join("file2.f90");
        let dir2 = root.join("dir2.f90");
        File::create(&file1)?;
        create_dir(&dir1)?;
        File::create(&file2)?;
        create_dir(&dir2)?;

        let paths = vec![file1.to_path_buf(), dir1.to_path_buf(), dir2.to_path_buf()];
        let extensions = vec!["f90".to_string(), "F90".to_string()];
        let include = convert_file_and_extensions_to_include(&Some(paths), &Some(extensions));

        let expected = Some(vec![
            format!("{}/*.{{f90,F90}}", dir1.to_string_lossy()),
            format!("{}/*.{{f90,F90}}", dir2.to_string_lossy()),
            file1.to_string_lossy().into_owned(),
        ]);
        assert_eq!(include, expected);

        Ok(())
    }

    #[test]
    fn convert_deprecated_default_file_and_extension() -> Result<()> {
        let extensions = vec!["f90".to_string(), "F90".to_string()];
        let include = convert_file_and_extensions_to_include(&None, &Some(extensions));

        let expected = Some(vec![("*.{f90,F90}").to_string()]);
        assert_eq!(include, expected);

        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn set_include_from_file_and_extension() {
        let options = Options {
            check: Some(CheckOptions {
                file_extensions: Some(vec!["f90".to_string(), "F90".to_string()]),
                ..CheckOptions::default()
            }),
            ..Options::default()
        };

        let root = Path::new("/some/abs/path/");
        let config = Configuration::from_options(options, root);

        let glob = "*.{f90,F90}";
        let expected = vec![FilePattern::User(
            glob.to_string(),
            GlobPath::normalize(glob, root),
        )];
        assert_eq!(config.include, Some(expected));
    }

    #[test]
    #[allow(deprecated)]
    fn dont_clobber_include_from_file_and_extension() {
        let options = Options {
            include: Some(vec!["*.f90".to_string(), "*.fpp".to_string()]),
            check: Some(CheckOptions {
                file_extensions: Some(vec!["f90".to_string(), "F90".to_string()]),
                ..CheckOptions::default()
            }),
        };

        let root = Path::new("/some/abs/path/");
        let config = Configuration::from_options(options, root);

        let expected = vec![
            FilePattern::User("*.f90".to_string(), GlobPath::normalize("*.f90", root)),
            FilePattern::User("*.fpp".to_string(), GlobPath::normalize("*.fpp", root)),
        ];
        assert_eq!(config.include, Some(expected));
    }
}
