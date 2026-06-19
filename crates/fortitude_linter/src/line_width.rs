use std::fmt;

use ruff_macros::CacheKey;
use serde::{Deserialize, Serialize};

/// The size of a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, CacheKey)]
pub struct IndentWidth(usize);

impl IndentWidth {
    pub(crate) fn as_usize(self) -> usize {
        self.0
    }
}

impl Default for IndentWidth {
    fn default() -> Self {
        Self(0usize)
    }
}

impl fmt::Display for IndentWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<usize> for IndentWidth {
    fn from(tab_size: usize) -> Self {
        Self(tab_size)
    }
}

impl From<IndentWidth> for usize {
    fn from(value: IndentWidth) -> Self {
        value.0
    }
}
