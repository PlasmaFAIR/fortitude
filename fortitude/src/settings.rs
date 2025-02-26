// Some parts adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

/// A collection of user-modifiable settings. Should be expanded as new features are added.
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use lazy_static::lazy_static;
use path_absolutize::path_dedot;
use ruff_diagnostics::Applicability;
use ruff_macros::CacheKey;
use serde::{de, Deserialize, Deserializer, Serialize};
use strum::IntoEnumIterator;

use crate::display_settings;
use crate::fs::{FilePatternSet, EXCLUDE_BUILTINS, FORTRAN_EXTS};
use crate::registry::Rule;
use crate::rule_selector::{CompiledPerFileIgnoreList, PreviewOptions, RuleSelector};
use crate::rule_table::RuleTable;

#[derive(Debug)]
pub struct Settings {
    pub check: CheckSettings,
    pub file_resolver: FileResolverSettings,
}

impl Default for Settings {
    fn default() -> Self {
        let project_root = path_dedot::CWD.as_path();
        Self {
            check: CheckSettings::new(project_root),
            file_resolver: FileResolverSettings::new(project_root),
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n# General Settings")?;
        display_settings! {
            formatter = f,
            fields = [
                self.check         | nested,
                self.file_resolver | nested,
            ]
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CheckSettings {
    pub project_root: PathBuf,

    pub rules: RuleTable,
    pub per_file_ignores: CompiledPerFileIgnoreList,

    pub line_length: usize,

    pub fix: bool,
    pub fix_only: bool,
    pub show_fixes: bool,
    pub unsafe_fixes: UnsafeFixes,
    pub output_format: OutputFormat,
    pub progress_bar: ProgressBar,
    pub preview: PreviewMode,
    pub ignore_allow_comments: IgnoreAllowComments,
}

impl CheckSettings {
    fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            rules: DEFAULT_SELECTORS
                .iter()
                .flat_map(|selector| selector.rules(&PreviewOptions::default()))
                .collect(),
            per_file_ignores: CompiledPerFileIgnoreList::default(),
            line_length: 100,
            fix: false,
            fix_only: false,
            show_fixes: false,
            unsafe_fixes: UnsafeFixes::default(),
            output_format: OutputFormat::default(),
            progress_bar: ProgressBar::default(),
            preview: PreviewMode::default(),
            ignore_allow_comments: IgnoreAllowComments::default(),
        }
    }
}

impl fmt::Display for CheckSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n# Check Settings")?;
        display_settings! {
            formatter = f,
            namespace = "check",
            fields = [
                self.project_root | path,
                self.rules | nested,
                self.per_file_ignores,
                self.line_length,
                self.fix,
                self.fix_only,
                self.show_fixes,
                self.output_format,
                self.progress_bar,
                self.preview,
            ]
        }
        Ok(())
    }
}
#[derive(Debug, CacheKey)]
pub struct FileResolverSettings {
    pub excludes: FilePatternSet,
    pub force_exclude: bool,
    pub files: Vec<PathBuf>,
    pub file_extensions: Vec<String>,
    pub respect_gitignore: bool,
    pub project_root: PathBuf,
}

impl fmt::Display for FileResolverSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n# File Resolver Settings")?;
        display_settings! {
            formatter = f,
            namespace = "file_resolver",
            fields = [
                self.excludes,
                self.force_exclude,
                self.files | paths,
                self.file_extensions | array,
                self.respect_gitignore,
                self.project_root | path,
            ]
        }
        Ok(())
    }
}

impl FileResolverSettings {
    fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            excludes: FilePatternSet::try_from_iter(EXCLUDE_BUILTINS.iter().cloned()).unwrap(),
            force_exclude: false,
            respect_gitignore: true,
            files: Vec::default(),
            file_extensions: FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect(),
        }
    }
}

/// Toggle for rules still in preview
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, CacheKey, is_macro::Is)]
pub enum PreviewMode {
    #[default]
    Disabled,
    Enabled,
}

impl From<bool> for PreviewMode {
    fn from(version: bool) -> Self {
        if version {
            PreviewMode::Enabled
        } else {
            PreviewMode::Disabled
        }
    }
}

impl fmt::Display for PreviewMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Enabled => write!(f, "enabled"),
        }
    }
}

lazy_static! {
    /// Default rule selection
    pub static ref DEFAULT_SELECTORS: Vec<RuleSelector> = {
        Rule::iter()
            .filter(Rule::is_default)
            .map(|rule| {
                let code = rule.noqa_code();
                let code = format!("{}{}", code.prefix(), code.suffix());
                RuleSelector::from_str(code.as_str()).unwrap()
            })
            .collect()
    };
}

/// Toggle for unsafe fixes.
/// `Hint` will not apply unsafe fixes but a message will be shown when they are available.
/// `Disabled` will not apply unsafe fixes or show a message.
/// `Enabled` will apply unsafe fixes.
#[derive(Debug, Copy, Clone, CacheKey, Default, PartialEq, Eq, is_macro::Is)]
pub enum UnsafeFixes {
    #[default]
    Hint,
    Disabled,
    Enabled,
}

impl fmt::Display for UnsafeFixes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Hint => "hint",
                Self::Disabled => "disabled",
                Self::Enabled => "enabled",
            }
        )
    }
}

impl From<bool> for UnsafeFixes {
    fn from(value: bool) -> Self {
        if value {
            UnsafeFixes::Enabled
        } else {
            UnsafeFixes::Disabled
        }
    }
}

impl UnsafeFixes {
    pub fn required_applicability(&self) -> Applicability {
        match self {
            Self::Enabled => Applicability::Unsafe,
            Self::Disabled | Self::Hint => Applicability::Safe,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Default, is_macro::Is)]
pub enum IgnoreAllowComments {
    Enabled,
    #[default]
    Disabled,
}

impl From<bool> for IgnoreAllowComments {
    fn from(value: bool) -> Self {
        if value {
            IgnoreAllowComments::Enabled
        } else {
            IgnoreAllowComments::Disabled
        }
    }
}

/// Command-line pattern for per-file rule ignores
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternPrefixPair {
    pub pattern: String,
    pub prefix: RuleSelector,
}

impl PatternPrefixPair {
    const EXPECTED_PATTERN: &'static str = "<FilePattern>:<RuleCode> pattern";
}

impl<'de> Deserialize<'de> for PatternPrefixPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_result = String::deserialize(deserializer)?;
        Self::from_str(str_result.as_str()).map_err(|_| {
            de::Error::invalid_value(
                de::Unexpected::Str(str_result.as_str()),
                &Self::EXPECTED_PATTERN,
            )
        })
    }
}

impl FromStr for PatternPrefixPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (pattern_str, code_string) = {
            let tokens = s.split(':').collect::<Vec<_>>();
            if tokens.len() != 2 {
                anyhow::bail!("Expected {}", Self::EXPECTED_PATTERN);
            }
            (tokens[0].trim(), tokens[1].trim())
        };
        let pattern = pattern_str.into();
        let prefix = RuleSelector::from_str(code_string)?;
        Ok(Self { pattern, prefix })
    }
}

/// Toggle for excluding files even when passed directly on the command line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, CacheKey, is_macro::Is)]
pub enum ExcludeMode {
    #[default]
    NoForce,
    Force,
}

impl From<bool> for ExcludeMode {
    fn from(b: bool) -> Self {
        if b {
            Self::Force
        } else {
            Self::NoForce
        }
    }
}

/// Toggle for respecting .gitignore files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, CacheKey, is_macro::Is)]
pub enum GitignoreMode {
    #[default]
    RespectGitignore,
    NoRespectGitignore,
}

impl From<bool> for GitignoreMode {
    fn from(value: bool) -> Self {
        if value {
            GitignoreMode::RespectGitignore
        } else {
            GitignoreMode::NoRespectGitignore
        }
    }
}

impl From<GitignoreMode> for bool {
    fn from(value: GitignoreMode) -> Self {
        match value {
            GitignoreMode::RespectGitignore => true,
            GitignoreMode::NoRespectGitignore => false,
        }
    }
}

/// Toggle for progress bar
#[derive(
    Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Hash, Default, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum ProgressBar {
    #[default]
    Off,
    Fancy,
    Ascii,
}

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Off => "off",
                Self::Fancy => "fancy",
                Self::Ascii => "ascii",
            }
        )
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Hash, Default, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Concise,
    #[default]
    Full,
    Json,
    JsonLines,
    Junit,
    Grouped,
    Github,
    Gitlab,
    Pylint,
    Rdjson,
    Azure,
    Sarif,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Concise => write!(f, "concise"),
            Self::Full => write!(f, "full"),
            Self::Json => write!(f, "json"),
            Self::JsonLines => write!(f, "json_lines"),
            Self::Junit => write!(f, "junit"),
            Self::Grouped => write!(f, "grouped"),
            Self::Github => write!(f, "github"),
            Self::Gitlab => write!(f, "gitlab"),
            Self::Pylint => write!(f, "pylint"),
            Self::Rdjson => write!(f, "rdjson"),
            Self::Azure => write!(f, "azure"),
            Self::Sarif => write!(f, "sarif"),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, is_macro::Is)]
pub enum FixMode {
    Generate,
    Apply,
    #[allow(dead_code)]
    Diff,
}

/// `display_settings!` is a macro that can display and format struct fields in a readable,
/// namespaced format. It's particularly useful at generating `Display` implementations
/// for types used in settings.
///
/// # Example
/// ```
/// use std::fmt;
/// use fortitude::display_settings;
/// #[derive(Default)]
/// struct Settings {
///     option_a: bool,
///     sub_settings: SubSettings,
///     option_b: String,
/// }
///
/// struct SubSettings {
///     name: String
/// }
///
/// impl Default for SubSettings {
///     fn default() -> Self {
///         Self { name: "Default Name".into() }
///     }
///
/// }
///
/// impl fmt::Display for SubSettings {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         display_settings! {
///             formatter = f,
///             namespace = "sub_settings",
///             fields = [
///                 self.name | quoted
///             ]
///         }
///         Ok(())
///     }
///
/// }
///
/// impl fmt::Display for Settings {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         display_settings! {
///             formatter = f,
///             fields = [
///                 self.option_a,
///                 self.sub_settings | nested,
///                 self.option_b | quoted,
///             ]
///         }
///         Ok(())
///     }
///
/// }
///
/// const EXPECTED_OUTPUT: &str = r#"option_a = false
/// sub_settings.name = "Default Name"
/// option_b = ""
/// "#;
///
/// fn main() {
///     let settings = Settings::default();
///     assert_eq!(format!("{settings}"), EXPECTED_OUTPUT);
/// }
/// ```
#[macro_export]
macro_rules! display_settings {
    (formatter = $fmt:ident, namespace = $namespace:literal, fields = [$($settings:ident.$field:ident $(| $modifier:tt)?),* $(,)?]) => {
        {
            const _PREFIX: &str = concat!($namespace, ".");
            $(
                display_settings!(@field $fmt, _PREFIX, $settings.$field $(| $modifier)?);
            )*
        }
    };
    (formatter = $fmt:ident, fields = [$($settings:ident.$field:ident $(| $modifier:tt)?),* $(,)?]) => {
        {
            const _PREFIX: &str = "";
            $(
                display_settings!(@field $fmt, _PREFIX, $settings.$field $(| $modifier)?);
            )*
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | debug) => {
        writeln!($fmt, "{}{} = {:?}", $prefix, stringify!($field), $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | path) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field.display())?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | quoted) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | globmatcher) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field.glob())?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | nested) => {
        write!($fmt, "{}", $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | optional) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            match &$settings.$field {
                Some(value) => writeln!($fmt, "{}", value)?,
                None        => writeln!($fmt, "none")?
            };
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | array) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in &$settings.$field {
                    writeln!($fmt, "\t{elem},")?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | map) => {
        {
            use itertools::Itertools;

            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "{{}}")?;
            } else {
                writeln!($fmt, "{{")?;
                for (key, value) in $settings.$field.iter().sorted_by(|(left, _), (right, _)| left.cmp(right)) {
                    writeln!($fmt, "\t{key} = {value},")?;
                }
                writeln!($fmt, "}}")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | set) => {
        {
            use itertools::Itertools;

            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in $settings.$field.iter().sorted_by(|left, right| left.cmp(right)) {
                    writeln!($fmt, "\t{elem},")?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | paths) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in &$settings.$field {
                    writeln!($fmt, "\t\"{}\",", elem.display())?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident) => {
        writeln!($fmt, "{}{} = {}", $prefix, stringify!($field), $settings.$field)?;
    };
}
