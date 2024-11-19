use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_text_size::TextRange;

use crate::settings::Settings;
use crate::PathRule;
use std::path::Path;

/// ## What it does
/// Checks for use of standard file extensions.
///
/// ## Why is it bad?
/// The standard file extensions for modern (free-form) Fortran are '.f90' or  '.F90'.
/// Forms that reference later Fortran standards such as '.f08' or '.F95' may be rejected
/// by some compilers and build tools.
#[violation]
pub struct NonStandardFileExtension {}

impl Violation for NonStandardFileExtension {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("file extension should be '.f90' or '.F90'")
    }
}

impl PathRule for NonStandardFileExtension {
    fn check(_settings: &Settings, path: &Path) -> Option<Diagnostic> {
        match path.extension() {
            Some(ext) => {
                // Must check like this as ext is an OsStr
                if ["f90", "F90"].iter().any(|&x| x == ext) {
                    None
                } else {
                    Some(Diagnostic::new(Self {}, TextRange::default()))
                }
            }
            None => Some(Diagnostic::new(Self {}, TextRange::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_file_extension() {
        let path = Path::new("my/dir/to/file.f95");
        assert_eq!(
            NonStandardFileExtension::check(&Settings::default(), path),
            Some(Diagnostic::new(
                NonStandardFileExtension {},
                TextRange::default()
            )),
        );
    }

    #[test]
    fn test_missing_file_extension() {
        let path = Path::new("my/dir/to/file");
        assert_eq!(
            NonStandardFileExtension::check(&Settings::default(), path),
            Some(Diagnostic::new(
                NonStandardFileExtension {},
                TextRange::default()
            )),
        );
    }

    #[test]
    fn test_correct_file_extensions() {
        let path1 = Path::new("my/dir/to/file.f90");
        let path2 = Path::new("my/dir/to/file.F90");
        assert_eq!(
            NonStandardFileExtension::check(&Settings::default(), path1),
            None
        );
        assert_eq!(
            NonStandardFileExtension::check(&Settings::default(), path2),
            None
        );
    }
}
