use crate::cli::CheckArgs;
use crate::fs::{FilePattern, FilePatternSet, EXCLUDE_BUILTINS, FORTRAN_EXTS};
use crate::options::Options;
use crate::registry::RuleNamespace;
use crate::rule_selector::{
    collect_per_file_ignores, CompiledPerFileIgnoreList, PerFileIgnore, PreviewOptions,
    RuleSelector, Specificity,
};
use crate::rule_table::RuleTable;
use crate::rules::Rule;
use crate::settings::{
    CheckSettings, ExcludeMode, FileResolverSettings, GitignoreMode, OutputFormat, PreviewMode,
    ProgressBar, Settings, UnsafeFixes, DEFAULT_SELECTORS,
};
use crate::{fs, warn_user_once_by_id, warn_user_once_by_message};

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::warn;
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
/// directory. Adapated from ruff
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
/// exists. Adapated from ruff
pub fn find_settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    for directory in path.as_ref().ancestors() {
        if let Some(settings) = settings_toml(directory)? {
            return Ok(Some(settings));
        }
    }
    Ok(None)
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
fn load_options<P: AsRef<Path>>(path: P) -> Result<Options> {
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

/// Read either fpm.toml or fortitude.toml into our "known good" file
/// settings struct
pub fn parse_config_file(config_file: &Option<PathBuf>) -> Result<Options> {
    let filename = match config_file {
        Some(filename) => filename.clone(),
        None => match find_settings_toml(path_absolutize::path_dedot::CWD.as_path())? {
            Some(filename) => filename,
            None => {
                return Ok(Options::default());
            }
        },
    };

    load_options(filename)
}

// This is our "known good" intermediate settings struct after we've
// read the config file, but before we've overridden it from the CLI
#[derive(Debug)]
pub struct Configuration {
    pub files: Vec<PathBuf>,
    pub ignore: Vec<RuleSelector>,
    pub select: Option<Vec<RuleSelector>>,
    pub extend_select: Vec<RuleSelector>,
    pub per_file_ignores: Option<Vec<PerFileIgnore>>,
    pub line_length: usize,
    pub file_extensions: Vec<String>,
    pub fix: bool,
    pub fix_only: bool,
    pub show_fixes: bool,
    pub unsafe_fixes: UnsafeFixes,
    pub output_format: OutputFormat,
    pub progress_bar: ProgressBar,
    pub preview: PreviewMode,
    pub exclude: Option<Vec<FilePattern>>,
    pub extend_exclude: Vec<FilePattern>,
    pub exclude_mode: ExcludeMode,
    pub gitignore_mode: GitignoreMode,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            files: Default::default(),
            ignore: Default::default(),
            select: Default::default(),
            extend_select: Default::default(),
            per_file_ignores: Default::default(),
            line_length: Settings::default().check.line_length,
            file_extensions: FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect(),
            fix: Default::default(),
            fix_only: Default::default(),
            show_fixes: Default::default(),
            unsafe_fixes: Default::default(),
            output_format: Default::default(),
            progress_bar: Default::default(),
            preview: Default::default(),
            exclude: Default::default(),
            extend_exclude: Default::default(),
            exclude_mode: Default::default(),
            gitignore_mode: Default::default(),
        }
    }
}

impl Configuration {
    /// Convert from config file options struct into our "known good" struct
    pub fn from_options(options: Options, project_root: &Path) -> Self {
        let check = options.check.unwrap_or_default();

        Self {
            files: check.files.unwrap_or_default(),
            ignore: check.ignore.unwrap_or_default(),
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
            line_length: check
                .line_length
                .unwrap_or(Settings::default().check.line_length),
            file_extensions: check
                .file_extensions
                .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect_vec()),
            fix: check.fix.unwrap_or_default(),
            fix_only: check.fix_only.unwrap_or_default(),
            show_fixes: check.show_fixes.unwrap_or_default(),
            unsafe_fixes: check
                .unsafe_fixes
                .map(UnsafeFixes::from)
                .unwrap_or_default(),
            output_format: check.output_format.unwrap_or_default(),
            progress_bar: check.progress_bar.unwrap_or_default(),
            preview: check.preview.map(PreviewMode::from).unwrap_or_default(),
            exclude: check.exclude.map(|paths| {
                paths
                    .into_iter()
                    .map(|pattern| {
                        let absolute = fs::normalize_path_to(&pattern, project_root);
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
                            let absolute = fs::normalize_path_to(&pattern, project_root);
                            FilePattern::User(pattern, absolute)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            exclude_mode: check
                .force_exclude
                .map(ExcludeMode::from)
                .unwrap_or_default(),
            gitignore_mode: check
                .respect_gitignore
                .map(GitignoreMode::from)
                .unwrap_or_default(),
        }
    }

    pub fn into_settings(self, project_root: &Path, args: &CheckArgs) -> Result<Settings> {
        let args = args.clone();

        let files = args.files.unwrap_or(self.files);
        let file_extensions = args.file_extensions.unwrap_or(self.file_extensions);

        let per_file_ignores = if let Some(per_file_ignores) = args.per_file_ignores {
            Some(collect_per_file_ignores(per_file_ignores))
        } else {
            self.per_file_ignores
        };

        let per_file_ignores = CompiledPerFileIgnoreList::resolve(
            per_file_ignores
                .unwrap_or_default()
                .into_iter()
                .chain(
                    args.extend_per_file_ignores
                        .map(collect_per_file_ignores)
                        .unwrap_or_default(),
                )
                .collect::<Vec<_>>(),
        )?;

        let exclude = FilePatternSet::try_from_iter(
            EXCLUDE_BUILTINS
                .iter()
                .cloned()
                .chain(
                    args.exclude
                        .unwrap_or(self.exclude.unwrap_or_default())
                        .into_iter(),
                )
                .chain(args.extend_exclude.unwrap_or_default().into_iter())
                .chain(self.extend_exclude.into_iter()),
        )?;

        let force_exclude = resolve_bool_arg(args.force_exclude, args.no_force_exclude)
            .map(ExcludeMode::from)
            .unwrap_or(self.exclude_mode);

        let respect_gitignore = resolve_bool_arg(args.respect_gitignore, args.no_respect_gitignore)
            .map(GitignoreMode::from)
            .unwrap_or(self.gitignore_mode);

        let preview = resolve_bool_arg(args.preview, args.no_preview)
            .map(PreviewMode::from)
            .unwrap_or(self.preview);

        let rule_selection = RuleSelection {
            select: args.select.or(self.select),
            // TODO: CLI ignore should _extend_ file ignore
            ignore: args.ignore.unwrap_or(self.ignore),
            extend_select: args.extend_select.unwrap_or(self.extend_select),
            fixable: None,
            unfixable: vec![],
            extend_fixable: vec![],
        };
        let rules = to_rule_table(rule_selection, &preview)?;

        let mut progress_bar = args.progress_bar.unwrap_or(self.progress_bar);
        // Override progress bar settings if not using colour terminal
        if progress_bar == ProgressBar::Fancy
            && !colored::control::SHOULD_COLORIZE.should_colorize()
        {
            progress_bar = ProgressBar::Ascii;
        }

        let output_format = args.output_format.unwrap_or(self.output_format);

        let show_fixes =
            resolve_bool_arg(args.show_fixes, args.no_show_fixes).unwrap_or(self.show_fixes);

        Ok(Settings {
            check: CheckSettings {
                project_root: project_root.to_path_buf(),
                rules,
                fix: resolve_bool_arg(args.fix, args.no_fix).unwrap_or(self.fix),
                fix_only: resolve_bool_arg(args.fix_only, args.no_fix_only)
                    .unwrap_or(self.fix_only),
                line_length: args.line_length.unwrap_or(self.line_length),
                unsafe_fixes: resolve_bool_arg(args.unsafe_fixes, args.no_unsafe_fixes)
                    .map(UnsafeFixes::from)
                    .unwrap_or(self.unsafe_fixes),
                preview,
                progress_bar,
                output_format,
                show_fixes,
                per_file_ignores,
                ignore_allow_comments: args.ignore_allow_comments.into(),
            },
            file_resolver: FileResolverSettings {
                project_root: project_root.to_path_buf(),
                excludes: exclude,
                files,
                file_extensions,
                respect_gitignore: respect_gitignore.is_respect_gitignore(),
                force_exclude: force_exclude.is_force(),
            },
        })
    }
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
        warn_user_once_by_id!(
            from,
            "`{from}` has been remapped to `{}{}`.",
            target.category().common_prefix(),
            target.short_code()
        );
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
                return Err(anyhow!("Selection of deprecated rule `{prefix}{code}` is not allowed when preview is enabled."));
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{registry::RuleSet, rule_selector::RuleSelector, settings::DEFAULT_SELECTORS};

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
}
