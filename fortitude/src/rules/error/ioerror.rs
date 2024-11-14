// Taken from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use crate::settings::Settings;
use crate::PathRule;
use std::path::Path;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

/// ## What it does
/// This is not a regular diagnostic; instead, it's raised when a file cannot be read
/// from disk.
///
/// ## Why is this bad?
/// An `IoError` indicates an error in the development setup. For example, the user may
/// not have permissions to read a given file, or the filesystem may contain a broken
/// symlink.
///
/// ## Example
/// On Linux or macOS:
/// ```shell
/// $ echo -e 'print*, "hello world!"\nend' > a.f90
/// $ chmod 000 a.f90
/// $ fortitude check a.f90
/// a.f90:1:1: E902 Permission denied (os error 13)
/// Found 1 error.
/// ```
///
/// ## References
/// - [UNIX Permissions introduction](https://mason.gmu.edu/~montecin/UNIXpermiss.htm)
/// - [Command Line Basics: Symbolic Links](https://www.digitalocean.com/community/tutorials/workflow-symbolic-links)
#[violation]
pub struct IoError {
    pub message: String,
}

/// E000
impl Violation for IoError {
    #[derive_message_formats]
    fn message(&self) -> String {
        let IoError { message } = self;
        format!("{message}")
    }
}

// Need to implement some kind of rule, although we only raise this manually
impl PathRule for IoError {
    fn check(_settings: &Settings, _path: &Path) -> Option<Diagnostic> {
        None
    }
}
