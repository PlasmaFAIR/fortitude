use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use fortitude_linter::warn_user_once;
use fortitude_workspace::resolver::{ConfigFile, ResolvedFile, fortran_files_in_path};
use itertools::Itertools;

use crate::cli::ConfigArguments;

pub(crate) fn show_files(
    files: &[PathBuf],
    fortitude_config: &ConfigFile,
    config_arguments: &ConfigArguments,
    writer: &mut impl Write,
) -> Result<()> {
    // Collect all paths
    let (paths, _resolver) = fortran_files_in_path(files, fortitude_config, config_arguments)?;

    if paths.is_empty() {
        warn_user_once!("No Fortran files found under the given path(s)");
        return Ok(());
    }

    for path in paths
        .into_iter()
        .flatten()
        .map(ResolvedFile::into_path)
        .sorted_unstable()
    {
        writeln!(writer, "{}", path.to_string_lossy())?;
    }

    Ok(())
}
