// Taken from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::borrow::Cow;

pub(crate) trait ShowNonprinting {
    fn show_nonprinting(&self) -> Cow<'_, str>;
}

macro_rules! impl_show_nonprinting {
    ($(($from:expr, $to:expr)),+) => {
        impl ShowNonprinting for str {
            fn show_nonprinting(&self) -> Cow<'_, str> {
                if self.find(&[$($from),*][..]).is_some() {
                    Cow::Owned(
                        self.$(replace($from, $to)).*
                    )
                } else {
                    Cow::Borrowed(self)
                }
            }
        }

        /// If `c` is an unprintable character, then this returns a printable
        /// representation of it (using a fancier Unicode codepoint).
        pub(crate) fn unprintable_replacement(c: char) -> Option<&'static str> {
            match c {
                $($from => Some($to),)*
                _ => None,
            }
        }
    };
}

impl_show_nonprinting!(('\x07', "␇"), ('\x08', "␈"), ('\x1b', "␛"), ('\x7f', "␡"));
