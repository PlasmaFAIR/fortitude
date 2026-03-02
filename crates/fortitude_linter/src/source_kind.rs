use std::{fmt::Formatter, path::Path};

use colored::Colorize;
use ruff_source_file::SourceFile;
use similar::{ChangeTag, TextDiff};

use crate::{fs, text_helpers::ShowNonprinting};

#[derive(Clone, Debug)]
pub struct SourceKindDiff<'a> {
    kind: DiffKind<'a>,
    path: Option<&'a Path>,
}

impl<'a> SourceKindDiff<'a> {
    pub fn new(src: &'a SourceFile, dst: &'a SourceFile, path: Option<&'a Path>) -> Self {
        Self {
            kind: DiffKind::Fortran(src.source_text(), dst.source_text()),
            path,
        }
    }
}

impl std::fmt::Display for SourceKindDiff<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            DiffKind::Fortran(original, modified) => {
                let mut diff = CodeDiff::new(original, modified);

                let relative_path = self.path.map(fs::relativize_path);

                if let Some(relative_path) = &relative_path {
                    diff.header(relative_path, relative_path);
                }

                writeln!(f, "{diff}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum DiffKind<'a> {
    Fortran(&'a str, &'a str),
}

struct CodeDiff<'a> {
    diff: TextDiff<'a, 'a, 'a, str>,
    header: Option<(&'a str, &'a str)>,
    missing_newline_hint: bool,
}

impl<'a> CodeDiff<'a> {
    fn new(original: &'a str, modified: &'a str) -> Self {
        let diff = TextDiff::from_lines(original, modified);
        Self {
            diff,
            header: None,
            missing_newline_hint: true,
        }
    }

    fn header(&mut self, original: &'a str, modified: &'a str) {
        self.header = Some((original, modified));
    }
}

impl std::fmt::Display for CodeDiff<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((original, modified)) = self.header {
            writeln!(f, "--- {}", original.show_nonprinting().red())?;
            writeln!(f, "+++ {}", modified.show_nonprinting().green())?;
        }

        let mut unified = self.diff.unified_diff();
        unified.missing_newline_hint(self.missing_newline_hint);

        // Individual hunks (section of changes)
        for hunk in unified.iter_hunks() {
            writeln!(f, "{}", hunk.header())?;

            // individual lines
            for change in hunk.iter_changes() {
                let value = change.value().show_nonprinting();
                match change.tag() {
                    ChangeTag::Equal => write!(f, " {value}")?,
                    ChangeTag::Delete => write!(f, "{}{}", "-".red(), value.red())?,
                    ChangeTag::Insert => write!(f, "{}{}", "+".green(), value.green())?,
                }

                if !self.diff.newline_terminated() {
                    writeln!(f)?;
                } else if change.missing_newline() {
                    if self.missing_newline_hint {
                        writeln!(f, "{}", "\n\\ No newline at end of file".red())?;
                    } else {
                        writeln!(f)?;
                    }
                }
            }
        }

        Ok(())
    }
}
