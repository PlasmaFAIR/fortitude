use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use globset::{Glob, GlobSet, GlobSetBuilder};
use path_absolutize::Absolutize;
use ruff_cache::{CacheKey, CacheKeyHasher};
use ruff_macros::CacheKey;
use serde::{Deserialize, Deserializer, de};

use crate::registry::Rule;
use crate::rule_selector::CompiledPerFileIgnoreList;

/// Wrapper around `std::fs::read_to_string` with some extra error
/// checking.
///
/// Check that the file length is representable as `u32` so
/// that we don't need to check when converting tree-sitter offsets
/// (usize) into ruff offsets (u32)
pub fn read_to_string(path: &Path) -> std::io::Result<String> {
    let metadata = path.metadata()?;
    let file_length = metadata.len();

    if TryInto::<u32>::try_into(file_length).is_err() {
        #[allow(non_snake_case)]
        let length_in_GiB = file_length as f64 / 1024.0 / 1024.0 / 1024.0;
        return Err(std::io::Error::other(format!(
            "larger than maximum 4 GiB ({length_in_GiB} GiB)"
        )));
    }
    std::fs::read_to_string(path)
}

/// Represents a path to be passed to [`Glob::new`].
#[derive(Debug, Clone, CacheKey, PartialEq, PartialOrd, Eq, Ord)]
pub struct GlobPath {
    path: PathBuf,
}

impl GlobPath {
    /// Constructs a [`GlobPath`] by escaping any glob metacharacters in `root` and normalizing
    /// `path` to the escaped `root`.
    ///
    /// See [`normalize_path_to`] for details of the normalization.
    pub fn normalize(path: impl AsRef<Path>, root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_string_lossy();
        let escaped = globset::escape(&root);
        let absolute = normalize_path_to(path, escaped);
        Self { path: absolute }
    }

    pub fn into_inner(self) -> PathBuf {
        self.path
    }
}

impl Deref for GlobPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, CacheKey)]
pub enum FilePattern {
    Builtin(&'static str),
    User(String, GlobPath),
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
        Ok(Self::User(
            s.to_string(),
            GlobPath::normalize(s, path_absolutize::path_dedot::CWD.as_path()),
        ))
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

impl Display for FilePattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                Self::Builtin(pattern) => pattern,
                Self::User(pattern, _) => pattern.as_str(),
            }
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilePatternSet {
    set: GlobSet,
    cache_key: u64,
    // This field is only for displaying the internals
    // of `set`.
    #[allow(clippy::used_underscore_binding)]
    _set_internals: Vec<FilePattern>,
}

impl Display for FilePatternSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self._set_internals.is_empty() {
            write!(f, "[]")?;
        } else {
            writeln!(f, "[")?;
            for pattern in &self._set_internals {
                writeln!(f, "\t{pattern},")?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl Deref for FilePatternSet {
    type Target = GlobSet;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl CacheKey for FilePatternSet {
    fn cache_key(&self, state: &mut CacheKeyHasher) {
        state.write_usize(self.set.len());
        state.write_u64(self.cache_key);
    }
}

impl FilePatternSet {
    pub fn try_from_iter<I>(patterns: I) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = FilePattern>,
    {
        let mut builder = GlobSetBuilder::new();
        let mut hasher = CacheKeyHasher::new();

        let mut _set_internals = vec![];

        for pattern in patterns {
            _set_internals.push(pattern.clone());
            pattern.cache_key(&mut hasher);
            pattern.add_to(&mut builder)?;
        }
        let set = builder.build()?;
        Ok(FilePatternSet {
            set,
            cache_key: hasher.finish(),
            _set_internals,
        })
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
pub fn ignores_from_path(path: &Path, ignore_list: &CompiledPerFileIgnoreList) -> Vec<Rule> {
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

/// Default paths to exclude when searching paths
pub static EXCLUDE_BUILTINS: &[FilePattern] = &[
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

/// Default paths to include
pub const INCLUDE: &[FilePattern] = &[
    FilePattern::Builtin("*.f90"),
    FilePattern::Builtin("*.F90"),
    FilePattern::Builtin("*.f95"),
    FilePattern::Builtin("*.F95"),
    FilePattern::Builtin("*.f03"),
    FilePattern::Builtin("*.F03"),
    FilePattern::Builtin("*.f08"),
    FilePattern::Builtin("*.F08"),
    FilePattern::Builtin("*.f18"),
    FilePattern::Builtin("*.F18"),
    FilePattern::Builtin("*.f23"),
    FilePattern::Builtin("*.F23"),
];
