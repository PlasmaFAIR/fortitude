use anyhow::Result;
use git2::{DiffDelta, DiffHunk, Repository};
use ruff_source_file::{LineIndex, OneIndexed, SourceFile};
use ruff_text_size::{TextRange, TextSize};
use serde::Deserialize;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::fs::{self};

/// Type exposed to user for input
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Deserialize)]
pub struct Filter {
    inner: Vec<LineFilter>,
}

impl FromStr for Filter {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: Vec<LineFilter> = serde_json::from_str(s)?;
        Ok(Self { inner })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Deserialize)]
pub struct LineFilter {
    name: PathBuf,
    lines: Option<Vec<LineRange>>,
}

impl FromStr for LineFilter {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Deserialize)]
pub struct LineRange {
    start: OneIndexed,
    end: OneIndexed,
}

impl FromStr for LineRange {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let array: [OneIndexed; 2] = serde_json::from_str(s)?;
        Ok(Self {
            start: array[0],
            end: array[0],
        })
    }
}

/// Type used internally for filtering
///
/// Maps filenames onto sets of offset ranges
#[derive(Debug, Clone)]
pub struct FilterMap {
    inner: HashMap<PathBuf, Vec<LineRange>>,
}

impl FilterMap {
    /// Construct a `FilterMap`, converting from line ranges to offset ranges
    pub fn new(value: Filter) -> Self {
        let inner = value
            .inner
            .into_iter()
            .map(|line_filter| {
                (
                    fs::normalize_path(line_filter.name),
                    line_filter.lines.unwrap_or_default(),
                )
            })
            .collect();
        Self { inner }
    }

    pub fn files(&self) -> Vec<&PathBuf> {
        self.inner.keys().collect()
    }

    pub fn contains<P: Into<PathBuf>>(&self, file: P) -> bool {
        self.inner.contains_key(&file.into())
    }

    /// Return offset ranges for filename, if it exists
    ///
    /// Requires a `SourceFile` to turn line numbers into offsets
    pub fn get<P: Into<PathBuf>>(&self, file: P, source: &SourceFile) -> Option<FilterSet> {
        self.inner
            .get(&file.into())
            .map(|range| FilterSet::from_source_file(range, source))
    }
}

/// Set of offset ranges into a file
#[derive(Debug)]
pub struct FilterSet {
    inner: Vec<TextRange>,
}

impl FilterSet {
    pub fn new(value: Vec<LineRange>, line_index: &LineIndex, contents: &str) -> Self {
        let inner: Vec<_> = value
            .into_iter()
            .map(|line_range| {
                let start = line_index.line_start(line_range.start, contents);
                let end = line_index.line_end(line_range.end, contents);

                TextRange::new(start, end)
            })
            .collect();
        Self { inner }
    }

    pub fn from_source_file(value: &[LineRange], source: &SourceFile) -> Self {
        let code = source.to_source_code();

        if value.is_empty() {
            return Self {
                inner: vec![TextRange::new(
                    TextSize::new(0),
                    TextSize::try_from(code.text().len()).unwrap(),
                )],
            };
        }

        let inner: Vec<_> = value
            .iter()
            .map(|line_range| {
                let start = code.line_start(line_range.start);
                let end = code.line_end(line_range.end);
                TextRange::new(start, end)
            })
            .collect();
        Self { inner }
    }

    /// Check if this `FilterSet` contains an offset.
    ///
    /// Ends of line ranges are considered included
    pub fn contains(&self, offset: TextSize) -> bool {
        self.inner
            .iter()
            .any(|range| range.contains_inclusive(offset))
    }

    pub fn contains_range(&self, other: TextRange) -> bool {
        self.inner.iter().any(|range| range.contains_range(other))
    }
}

/// Create a line filter for just the files currently staged in the index
pub fn git_staged_files<P: AsRef<Path>>(project_root: P) -> Result<FilterMap> {
    let repo = Repository::open_from_env()?;

    let head_tree = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head_tree), None, None)?;

    // Bit annoying, but the `FnMut` trait is still nightly, so we can't
    // implement it directly on `HunkCb`, so we need a closure that captures our
    // state and then calls it
    let mut hunk_state = HunkCb::new(project_root.as_ref());
    let mut hunk_cb =
        |diff_delta: DiffDelta, hunk: DiffHunk| -> bool { hunk_state.call(diff_delta, hunk) };

    diff.foreach(&mut file_cb, None, Some(&mut hunk_cb), None)?;

    let ls = std::process::Command::new("ls").output()?;
    log::debug!("ls: {:?}", str::from_utf8(&ls.stdout)?);
    let pwd = std::process::Command::new("pwd").output()?;
    log::debug!("pwd: {:?}", str::from_utf8(&pwd.stdout)?);

    Ok(hunk_state.filter)
}

/// Create a filter for files that have changed since ``treeish``
pub fn git_since<P: AsRef<Path>>(treeish: &str, project_root: P) -> Result<FilterMap> {
    let repo = Repository::open_from_env()?;

    // TODO(peter): nicer error message if treeish isn't correct
    let object = repo.revparse_single(treeish)?;
    let tree = object.peel_to_tree()?;
    let diff = repo.diff_tree_to_workdir(Some(&tree), None)?;

    let mut hunk_state = HunkCb::new(project_root.as_ref());
    let mut hunk_cb =
        |diff_delta: DiffDelta, hunk: DiffHunk| -> bool { hunk_state.call(diff_delta, hunk) };

    diff.foreach(&mut file_cb, None, Some(&mut hunk_cb), None)?;

    let ls = std::process::Command::new("ls").output()?;
    log::debug!("ls: {:?}", str::from_utf8(&ls.stdout)?);
    let pwd = std::process::Command::new("pwd").output()?;
    log::debug!("pwd: {:?}", str::from_utf8(&pwd.stdout)?);

    Ok(hunk_state.filter)
}

/// Callback for hunks with `Diff::foreach`
///
/// Let's us reuse the same closure between the `git_staged_files` and
/// `git_since`
struct HunkCb<'a> {
    filter: FilterMap,
    project_root: &'a Path,
}

impl<'a> HunkCb<'a> {
    fn new(project_root: &'a Path) -> Self {
        let inner = HashMap::new();
        let filter = FilterMap { inner };
        Self {
            filter,
            project_root,
        }
    }

    fn call(&mut self, diff_delta: DiffDelta, hunk: DiffHunk) -> bool {
        if let Some(file) = diff_delta.new_file().path() {
            let start = OneIndexed::from_zero_indexed(hunk.new_start() as usize);
            let end = OneIndexed::from_zero_indexed((hunk.new_start() + hunk.new_lines()) as usize);
            let range = LineRange { start, end };
            let file_abs = fs::normalize_path_to(file, self.project_root);
            log::debug!("file: {}", file.display());
            log::debug!("project_root: {}", self.project_root.display());
            log::debug!("file_abs: {}", file_abs.display());
            self.filter
                .inner
                .entry(file_abs)
                .and_modify(|f| f.push(range.clone()))
                .or_insert(vec![range]);
        }

        true
    }
}

fn file_cb(_diff_delta: DiffDelta, _progress: f32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::{Context, Result};
    use lazy_static::lazy_static;
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::TextSize;

    const TEST_FILE: &str = r#"line 1
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
line 10
"#;

    lazy_static! {
        static ref SOURCE_FILE: SourceFile =
            SourceFileBuilder::new("file1.f90", TEST_FILE).finish();
    }

    fn make_filter() -> Filter {
        Filter {
            inner: vec![
                LineFilter {
                    name: "file1.f90".into(),
                    lines: Some(vec![
                        LineRange {
                            start: OneIndexed::new(1).unwrap(),
                            end: OneIndexed::new(3).unwrap(),
                        },
                        LineRange {
                            start: OneIndexed::new(8).unwrap(),
                            end: OneIndexed::new(10).unwrap(),
                        },
                    ]),
                },
                LineFilter {
                    name: "file2.f90".into(),
                    lines: None,
                },
            ],
        }
    }

    #[test]
    fn from_string() -> Result<()> {
        let example = r#"[
  {"name": "file1.f90", "lines": [[1, 3], [8, 10]]},
  {"name": "file2.f90"}
]"#;

        let filter = Filter::from_str(example)?;

        assert_eq!(filter, make_filter());

        Ok(())
    }

    #[test]
    fn filter_map() -> Result<()> {
        let filter = FilterMap::new(make_filter());

        assert!(
            filter
                .get(fs::normalize_path("file1.f90"), &SOURCE_FILE)
                .is_some()
        );
        assert!(filter.get("nothing", &SOURCE_FILE).is_none());
        Ok(())
    }

    #[test]
    fn filter_set() -> Result<()> {
        let filter = FilterMap::new(make_filter());
        let filter = filter
            .get(fs::normalize_path("file1.f90"), &SOURCE_FILE)
            .context("Expected file1.f90")?;

        assert!(filter.contains(TextSize::new(18)));
        assert!(filter.contains(TextSize::new(52)));
        assert!(!filter.contains(TextSize::new(25)));
        assert!(!filter.contains(TextSize::new(129)));

        assert!(filter.contains_range(TextRange::new(TextSize::new(6), TextSize::new(17))));
        assert!(!filter.contains_range(TextRange::new(TextSize::new(32), TextSize::new(38))));

        Ok(())
    }
}
