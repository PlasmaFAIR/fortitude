use std::path::{Path, PathBuf};
use std::str::FromStr;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{types::TypesBuilder, WalkBuilder};
use itertools::Itertools;
use log::debug;
use path_absolutize::Absolutize;
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::registry::Rule;
use crate::rule_selector::CompiledPerFileIgnoreList;
use crate::settings::{ExcludeMode, GitignoreMode};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize)]
pub enum FilePattern {
    Builtin(&'static str),
    User(String, PathBuf),
}

impl FilePattern {
    const EXPECTED_PATTERN: &'static str = "<FilePattern>";

    pub fn add_to(self, builder: &mut GlobSetBuilder) -> anyhow::Result<()> {
        match self {
            FilePattern::Builtin(pattern) => {
                builder.add(Glob::from_str(pattern)?);
            }
            FilePattern::User(pattern, absolute) => {
                // Add the absolute path.
                builder.add(Glob::new(&absolute.to_string_lossy())?);

                // Add basename path.
                if !pattern.contains(std::path::MAIN_SEPARATOR) {
                    builder.add(Glob::new(&pattern)?);
                }
            }
        }
        Ok(())
    }
}

impl FromStr for FilePattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = s.to_string();
        let absolute = normalize_path(&pattern);
        Ok(Self::User(pattern, absolute))
    }
}

impl<'de> Deserialize<'de> for FilePattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_result = String::deserialize(deserializer)?;
        Self::from_str(str_result.as_str()).map_err(|_| {
            de::Error::invalid_value(
                de::Unexpected::Str(str_result.as_str()),
                &Self::EXPECTED_PATTERN,
            )
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilePatternSet {
    pub set: GlobSet,
}

impl FilePatternSet {
    pub fn try_from_iter<I>(patterns: I) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = FilePattern>,
    {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            pattern.add_to(&mut builder)?;
        }
        let set = builder.build()?;
        Ok(FilePatternSet { set })
    }

    pub fn matches<P: AsRef<Path>>(&self, path: P) -> bool {
        match std::path::absolute(path.as_ref()) {
            Ok(path) => match path.clone().file_name() {
                Some(basename) => self.set.is_match(path) || self.set.is_match(basename),
                None => false,
            },
            _ => false,
        }
    }

    pub fn ancestor_matches<P: AsRef<Path>, R: AsRef<Path>>(
        &self,
        path: P,
        project_root: R,
    ) -> bool {
        let project_root = project_root.as_ref();
        match std::path::absolute(path.as_ref()) {
            Ok(path) => path
                .ancestors()
                .take_while(|ancestor| *ancestor != project_root)
                .any(|ancestor| match ancestor.file_name() {
                    Some(basename) => self.set.is_match(ancestor) || self.set.is_match(basename),
                    None => false,
                }),
            _ => false,
        }
    }
}

/// Create a set with codes matching the pattern/code pairs.
pub(crate) fn ignores_from_path(path: &Path, ignore_list: &CompiledPerFileIgnoreList) -> Vec<Rule> {
    let file_name = path.file_name().expect("Unable to parse filename");
    ignore_list
        .iter()
        .filter_map(|entry| {
            if entry.basename_matcher.is_match(file_name) || entry.absolute_matcher.is_match(path) {
                if entry.negated {
                    None
                } else {
                    Some(&entry.rules)
                }
            } else if entry.negated {
                Some(&entry.rules)
            } else {
                None
            }
        })
        .flatten()
        .copied()
        .collect()
}

/// Convert any path to an absolute path (based on the current working
/// directory).
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize() {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

/// Convert any path to an absolute path (based on the specified project root).
pub fn normalize_path_to<P: AsRef<Path>, R: AsRef<Path>>(path: P, project_root: R) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize_from(project_root.as_ref()) {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

/// Convert an absolute path to be relative to the current working directory.
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    #[cfg(target_arch = "wasm32")]
    let cwd = Path::new(".");
    #[cfg(not(target_arch = "wasm32"))]
    let cwd = path_absolutize::path_dedot::CWD.as_path();

    if let Ok(path) = path.strip_prefix(cwd) {
        return format!("{}", path.display());
    }
    format!("{}", path.display())
}

/// Convert an absolute path to be relative to the specified project root.
#[allow(dead_code)]
pub fn relativize_path_to<P: AsRef<Path>, R: AsRef<Path>>(path: P, project_root: R) -> String {
    format!(
        "{}",
        pathdiff::diff_paths(&path, project_root)
            .expect("Could not diff paths")
            .display()
    )
}

/// Default extensions to check
pub const FORTRAN_EXTS: &[&str] = &[
    "f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23",
];

// Default paths to exclude when searching paths
pub(crate) static EXCLUDE_BUILTINS: &[FilePattern] = &[
    FilePattern::Builtin(".git"),
    FilePattern::Builtin(".git-rewrite"),
    FilePattern::Builtin(".hg"),
    FilePattern::Builtin(".svn"),
    FilePattern::Builtin("venv"),
    FilePattern::Builtin(".venv"),
    FilePattern::Builtin("pyenv"),
    FilePattern::Builtin(".pyenv"),
    FilePattern::Builtin(".eggs"),
    FilePattern::Builtin("site-packages"),
    FilePattern::Builtin(".vscode"),
    FilePattern::Builtin("build"),
    FilePattern::Builtin("_build"),
    FilePattern::Builtin("dist"),
    FilePattern::Builtin("_dist"),
];

/// Expand the input list of files to include all Fortran files.
pub fn get_files<P: AsRef<Path>, R: AsRef<Path>, S: AsRef<str>>(
    paths: &[P],
    project_root: R,
    extensions: &[S],
    excludes: FilePatternSet,
    exclude_mode: ExcludeMode,
    gitignore_mode: GitignoreMode,
) -> anyhow::Result<Vec<PathBuf>> {
    debug!("Gathering files");
    let project_root = project_root.as_ref().to_path_buf();
    debug!("Project root: {:?}", project_root);
    // Normalise all paths and remove duplicates.
    // If exclude_mode is set to Force, remove paths that match the exclude patterns.
    let paths: Vec<_> = if matches!(exclude_mode, ExcludeMode::Force) {
        let (excluded, paths): (Vec<_>, Vec<_>) = paths
            .iter()
            .map(normalize_path)
            .unique()
            .partition(|p| excludes.ancestor_matches(p, &project_root));
        if !excluded.is_empty() {
            debug!("Force excluded paths: {:?}", excluded);
        }
        paths
    } else {
        paths.iter().map(normalize_path).unique().collect()
    };
    debug!("Paths provided: {:?}", paths);

    // The remaining non-directory paths are always included; split into directories and files.
    // Note that this includes paths that do not exist, as these should be reported to the user.
    let (dirs, files): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| p.is_dir());

    // Collect all files from directories
    let dir_contents = if let Some((first_dir, rest)) = dirs.split_first() {
        // Create a directory walker that follows exclude patterns
        let mut builder = WalkBuilder::new(first_dir);
        for path in rest {
            builder.add(path);
        }
        builder.standard_filters(gitignore_mode.into());
        builder.hidden(false);
        builder.filter_entry(move |e| !excludes.matches(e.path()));

        // Add file type filter for provided file extensions
        // Directories will be skipped
        let mut file_types = TypesBuilder::new();
        for ext in extensions {
            file_types.add(ext.as_ref(), format!("*.{}", ext.as_ref()).as_str())?;
        }
        file_types.select("all");
        builder.types(file_types.build()?);

        // Collect all valid files from directories
        builder
            .build()
            .filter_map(|p| p.ok()) // skip dirs if user doesn't have permission
            .map(|p| p.into_path())
            .filter(|p| !p.is_dir())
            .collect()
    } else {
        // No dirs remain after removing excludes and splitting into dirs and files
        vec![]
    };

    // Return all files found
    Ok(files.into_iter().chain(dir_contents).collect())
}
