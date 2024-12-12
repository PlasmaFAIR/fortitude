use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A collection of user-modifiable settings. Should be expanded as new features are added.
use crate::rule_selector::RuleSelector;
use ruff_diagnostics::Applicability;
use ruff_macros::CacheKey;
use serde::{de, Deserialize, Deserializer, Serialize};

pub struct Settings {
    pub line_length: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { line_length: 100 }
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

impl Display for PreviewMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Enabled => write!(f, "enabled"),
        }
    }
}

/// Default rule selection
pub const DEFAULT_SELECTORS: &[RuleSelector] = &[RuleSelector::All];

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

impl Display for UnsafeFixes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl Display for ProgressBar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
