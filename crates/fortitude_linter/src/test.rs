use std::path::Path;

use anyhow::Result;
use ruff_diagnostics::Applicability;
use ruff_source_file::{SourceFile, SourceFileBuilder};

use crate::{
    check_file,
    diagnostics::{DisplayDiagnosticConfig, DisplayDiagnostics},
    fs::read_to_string,
    settings::{self, CheckSettings, FixMode, OutputFormat},
};

#[macro_export]
macro_rules! apply_common_filters {
    {} => {
        let mut settings = insta::Settings::clone_current();
        // Convert windows paths to Unix Paths.
        settings.add_filter(r"\\\\?([\w\d.])", "/$1");
        let _bound = settings.bind_to_scope();
    }
}

pub(crate) fn test_resource_path(path: impl AsRef<Path>) -> std::path::PathBuf {
    Path::new("./resources/test/").join(path)
}

/// Run [`check_path`] on a Fortran file in the `resources/test/fixtures` directory.
pub(crate) fn test_path(path: impl AsRef<Path>, settings: &CheckSettings) -> Result<String> {
    let path = test_resource_path("fixtures").join(path);
    let source = read_to_string(&path)?;
    let filename = path.to_string_lossy();
    let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();
    Ok(test_contents(&path, &file, settings))
}

pub(crate) fn test_contents(path: &Path, file: &SourceFile, settings: &CheckSettings) -> String {
    match check_file(
        path,
        file,
        &None,
        settings,
        FixMode::Generate,
        settings::IgnoreAllowComments::Disabled,
    ) {
        Ok(diagnostics) => {
            if diagnostics.messages.is_empty() {
                return String::new();
            }

            let config = DisplayDiagnosticConfig::new()
                .format(OutputFormat::Full)
                .hide_severity(true)
                .with_show_fix_status(true)
                .show_fix_diff(true)
                .with_fix_applicability(Applicability::DisplayOnly);

            DisplayDiagnostics::new(&config, &diagnostics.messages).to_string()
        }
        Err(msg) => {
            panic!("Failed to process: {msg}");
        }
    }
}
