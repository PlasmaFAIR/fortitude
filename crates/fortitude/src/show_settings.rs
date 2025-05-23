use std::io::Write;

use anyhow::Result;

use crate::settings::Settings;

pub(crate) fn show_settings(settings: &Settings, writer: &mut impl Write) -> Result<()> {
    writeln!(
        writer,
        "Resolved settings for \"{}\"",
        settings.check.project_root.display()
    )?;
    write!(writer, "{settings}")?;
    Ok(())
}
