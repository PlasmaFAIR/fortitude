use std::{io::Write, path::PathBuf};

use anyhow::{Result, bail};

use fortitude_workspace::resolver::{ConfigFile, ResolvedFile, fortran_files_in_path};
use itertools::Itertools;

use crate::cli::ConfigArguments;

pub(crate) fn show_settings(
    files: &[PathBuf],
    fortitude_config: &ConfigFile,
    config_arguments: &ConfigArguments,
    writer: &mut impl Write,
) -> Result<()> {
    // Collect all paths
    let (paths, resolver) = fortran_files_in_path(files, fortitude_config, config_arguments)?;

    // Get the "first" file
    let Some(path) = paths
        .into_iter()
        .flatten()
        .map(ResolvedFile::into_path)
        .sorted_unstable()
        .next()
    else {
        bail!("No files found under the given path");
    };

    let settings = resolver.resolve(&path);

    writeln!(writer, "Resolved settings for \"{}\"", path.display())?;
    if let Some(settings_path) = fortitude_config.path.as_ref() {
        writeln!(writer, "Settings path: \"{}\"", settings_path.display())?;
    }
    write!(writer, "{settings}")?;
    Ok(())
}
