use std::path::Path;

use anyhow::Result;
use itertools::Itertools;
use ruff_source_file::{SourceFile, SourceFileBuilder};

use crate::{
    check::{
        ast_entrypoint_map, check_file, read_to_string, rules_to_path_rules, rules_to_text_rules,
    },
    message::{DiagnosticMessage, Emitter, TextEmitter},
    rules::Rule,
    settings::Settings,
};

pub(crate) fn test_resource_path(path: impl AsRef<Path>) -> std::path::PathBuf {
    Path::new("./resources/test/").join(path)
}

/// Run [`check_path`] on a Fortran file in the `resources/test/fixtures` directory.
pub(crate) fn test_path(
    path: impl AsRef<Path>,
    rules: &[Rule],
    settings: &Settings,
) -> Result<String> {
    let path = test_resource_path("fixtures").join(path);
    let source = read_to_string(&path)?;
    let filename = path.to_string_lossy();
    let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();
    Ok(test_contents(&path, &file, rules, settings))
}

pub(crate) fn test_contents(
    path: &Path,
    file: &SourceFile,
    rules: &[Rule],
    settings: &Settings,
) -> String {
    let path_rules = rules_to_path_rules(rules);
    let text_rules = rules_to_text_rules(rules);
    let ast_entrypoints = ast_entrypoint_map(rules);

    match check_file(
        &path_rules,
        &text_rules,
        &ast_entrypoints,
        path,
        file,
        settings,
    ) {
        Ok(violations) => {
            if violations.is_empty() {
                return String::new();
            }
            let diagnostics = violations
                .into_iter()
                .map(|v| DiagnosticMessage::from_ruff(file, v))
                .collect_vec();

            let mut output = Vec::new();

            TextEmitter::default()
                .with_show_fix_status(true)
                .with_show_fix_diff(true)
                .with_show_source(true)
                .with_unsafe_fixes(crate::settings::UnsafeFixes::Enabled)
                .emit(&mut output, &diagnostics)
                .unwrap();

            String::from_utf8(output).unwrap()
        }
        Err(msg) => {
            panic!("Failed to process: {msg}");
        }
    }
}
