use crate::rules::{Code, Violation};
use std::path::Path;
/// Defines rule that enforces use of standard file extensions.

pub const USE_STANDARD_FILE_EXTENSIONS: &str = "\
    The standard file extensions for modern (free-form) Fortran are '.f90' or '.F90'.
    Forms that reference later Fortran standards such as '.f08' or '.F95' may be
    rejected by some compilers and build tools.";

fn file_extension_violation(path: &Path, code: Code) -> Violation {
    Violation::new(path, 0, code, "file extension should be '.f90' or '.F90'")
}

pub fn use_standard_file_extensions(code: Code, path: &Path) -> Vec<Violation> {
    match path.extension() {
        Some(ext) => {
            if ["f90", "F90"].iter().any(|&x| x == ext) {
                vec![]
            } else {
                vec![file_extension_violation(path, code)]
            }
        }
        None => vec![file_extension_violation(path, code)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::TEST_CODE;

    #[test]
    fn test_bad_file_extension() {
        let path = Path::new("my/dir/to/file.f95");
        assert_eq!(
            use_standard_file_extensions(TEST_CODE, &path),
            vec![file_extension_violation(&path, TEST_CODE)],
        );
    }

    #[test]
    fn test_missing_file_extension() {
        let path = Path::new("my/dir/to/file");
        assert_eq!(
            use_standard_file_extensions(TEST_CODE, &path),
            vec![file_extension_violation(&path, TEST_CODE)],
        );
    }

    #[test]
    fn test_correct_file_extensions() {
        let path1 = Path::new("my/dir/to/file.f90");
        let path2 = Path::new("my/dir/to/file.F90");
        assert_eq!(use_standard_file_extensions(TEST_CODE, &path1), vec![]);
        assert_eq!(use_standard_file_extensions(TEST_CODE, &path2), vec![]);
    }
}
