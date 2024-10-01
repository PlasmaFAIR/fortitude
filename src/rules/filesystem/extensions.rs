use crate::settings::Settings;
use crate::violation;
use crate::{PathRule, Rule, Violation};
use std::path::Path;
/// Defines rule that enforces use of standard file extensions.

pub struct NonStandardFileExtension {}

impl Rule for NonStandardFileExtension {
    fn new(_settings: &Settings) -> Self {
        NonStandardFileExtension {}
    }

    fn explain(&self) -> &'static str {
        "
        The standard file extensions for modern (free-form) Fortran are '.f90' or  '.F90'.
        Forms that reference later Fortran standards such as '.f08' or '.F95' may be rejected
        by some compilers and build tools.
        "
    }
}

impl PathRule for NonStandardFileExtension {
    fn check(&self, path: &Path) -> Option<Violation> {
        let msg: &str = "file extension should be '.f90' or '.F90'";
        match path.extension() {
            Some(ext) => {
                // Must check like this as ext is an OsStr
                if ["f90", "F90"].iter().any(|&x| x == ext) {
                    None
                } else {
                    Some(violation!(msg))
                }
            }
            None => Some(violation!(msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::default_settings;
    use crate::violation;

    #[test]
    fn test_bad_file_extension() {
        let path = Path::new("my/dir/to/file.f95");
        let rule = NonStandardFileExtension::new(&default_settings());
        assert_eq!(
            rule.check(path),
            Some(violation!["file extension should be '.f90' or '.F90'"]),
        );
    }

    #[test]
    fn test_missing_file_extension() {
        let path = Path::new("my/dir/to/file");
        let rule = NonStandardFileExtension::new(&default_settings());
        assert_eq!(
            rule.check(path),
            Some(violation!["file extension should be '.f90' or '.F90'"]),
        );
    }

    #[test]
    fn test_correct_file_extensions() {
        let path1 = Path::new("my/dir/to/file.f90");
        let path2 = Path::new("my/dir/to/file.F90");
        let rule = NonStandardFileExtension::new(&default_settings());
        assert_eq!(rule.check(path1), None);
        assert_eq!(rule.check(path2), None);
    }
}
