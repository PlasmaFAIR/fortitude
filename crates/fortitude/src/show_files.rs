use std::io::Write;

use anyhow::Result;
use itertools::Itertools;

use fortitude_linter::fs::get_files;
use fortitude_linter::settings::FileResolverSettings;

pub(crate) fn show_files(
    resolver: &FileResolverSettings,
    is_stdin: bool,
    writer: &mut impl Write,
) -> Result<()> {
    let files = get_files(resolver, is_stdin);

    for file in files.into_iter().flatten().sorted_unstable() {
        writeln!(writer, "{}", file.to_string_lossy())?;
    }

    Ok(())
}
