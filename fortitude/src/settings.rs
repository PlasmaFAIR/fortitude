use ruff_macros::CacheKey;
/// A collection of user-modifiable settings. Should be expanded as new features are added.
use std::fmt::{Display, Formatter};

use crate::rule_selector::RuleSelector;

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
