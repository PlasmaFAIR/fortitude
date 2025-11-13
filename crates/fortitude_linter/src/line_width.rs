use std::{fmt, num::NonZeroU8};

use ruff_macros::CacheKey;
use serde::{Deserialize, Serialize};

/// The size of a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, CacheKey)]
pub struct IndentWidth(NonZeroU8);

impl IndentWidth {
    pub(crate) fn as_usize(self) -> usize {
        self.0.get() as usize
    }
}

impl Default for IndentWidth {
    fn default() -> Self {
        Self(NonZeroU8::new(4).unwrap())
    }
}

impl fmt::Display for IndentWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<NonZeroU8> for IndentWidth {
    fn from(tab_size: NonZeroU8) -> Self {
        Self(tab_size)
    }
}

impl From<IndentWidth> for NonZeroU8 {
    fn from(value: IndentWidth) -> Self {
        value.0
    }
}
