use crate::{
    CheckContext,
    diagnostics::{Diagnostic, Violation},
};
use fortitude_macros::ViolationMetadata;
use ruff_macros::derive_message_formats;
use ruff_text_size::TextRange;

use std::path::Path;

/// ## What it does
/// Checks for use of standard file extensions.
///
/// ## Why is it bad?
/// The standard file extensions for modern (free-form) Fortran are '.f90' or  '.F90'.
/// Forms that reference later Fortran standards such as '.f08' or '.F95' may be rejected
/// by some compilers and build tools.
///
/// The file extension '.pf' is permitted for users of the pFUnit testing framework.
#[derive(ViolationMetadata)]
pub(crate) struct NonStandardFileExtension {}

impl Violation for NonStandardFileExtension {
    #[derive_message_formats]
    fn message(&self) -> String {
        "file extension should be '.f90' or '.F90'".to_string()
    }
}

impl NonStandardFileExtension {
    pub fn check(context: &CheckContext) -> Option<Diagnostic> {
        let path = Path::new(context.file.name());
        match path.extension() {
            Some(ext) => {
                // Must check like this as ext is an OsStr
                if ["f90", "F90", "pf"].iter().any(|&x| x == ext) {
                    None
                } else {
                    Some(context.create_diagnostic(Self {}, TextRange::default()))
                }
            }
            None => Some(context.create_diagnostic(Self {}, TextRange::default())),
        }
    }
}
