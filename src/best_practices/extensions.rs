use crate::core::{Method, Rule, Violation};
use crate::violation;
use std::path::Path;
/// Defines rule that enforces use of standard file extensions.

fn use_standard_file_extensions(path: &Path) -> Option<Violation> {
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

pub struct UseStandardFileExtensions {}

impl Rule for UseStandardFileExtensions {
    fn method(&self) -> Method {
        Method::Path(Box::new(use_standard_file_extensions))
    }

    fn explain(&self) -> &str {
        "
        The standard file extensions for modern (free-form) Fortran are '.f90' or  '.F90'.
        Forms that reference later Fortran standards such as '.f08' or '.F95' may be rejected
        by some compilers and build tools.
        "
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::violation;

    #[test]
    fn test_bad_file_extension() {
        let path = Path::new("my/dir/to/file.f95");
        assert_eq!(
            use_standard_file_extensions(&path),
            Some(violation!["file extension should be '.f90' or '.F90'"]),
        );
    }

    #[test]
    fn test_missing_file_extension() {
        let path = Path::new("my/dir/to/file");
        assert_eq!(
            use_standard_file_extensions(&path),
            Some(violation!["file extension should be '.f90' or '.F90'"]),
        );
    }

    #[test]
    fn test_correct_file_extensions() {
        let path1 = Path::new("my/dir/to/file.f90");
        let path2 = Path::new("my/dir/to/file.F90");
        assert_eq!(use_standard_file_extensions(&path1), None);
        assert_eq!(use_standard_file_extensions(&path2), None);
    }
}
