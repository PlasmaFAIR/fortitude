use std::{collections::HashMap, path::PathBuf, str::FromStr};

use path_absolutize::Absolutize;
use ruff_source_file::{LineIndex, OneIndexed, SourceFile};
use ruff_text_size::{TextRange, TextSize};
use serde::Deserialize;

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
                    // Make the path name absolute, or fall back to given path
                    line_filter
                        .name
                        .absolutize()
                        .map(|file| file.into_owned())
                        .unwrap_or(line_filter.name),
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

#[cfg(test)]
mod tests {
    use std::path::Path;

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
                .get(Path::new("file1.f90").absolutize()?, &SOURCE_FILE)
                .is_some()
        );
        assert!(filter.get("nothing", &SOURCE_FILE).is_none());
        Ok(())
    }

    #[test]
    fn filter_set() -> Result<()> {
        let filter = FilterMap::new(make_filter());
        let filter = filter
            .get(Path::new("file1.f90").absolutize()?, &SOURCE_FILE)
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
