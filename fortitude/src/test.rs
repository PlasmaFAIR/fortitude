use std::path::Path;

use anyhow::Result;
use ruff_source_file::{SourceFile, SourceFileBuilder};

use crate::{
    check::{
        ast_entrypoint_map, check_file, read_to_string, rules_to_path_rules, rules_to_text_rules,
    },
    message::{Emitter, TextEmitter},
    rule_table::RuleTable,
    rules::Rule,
    settings::{self, FixMode, Settings},
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
    let rule_table = RuleTable::from_iter(rules.iter().cloned());
    let path_rules = rules_to_path_rules(&rule_table);
    let text_rules = rules_to_text_rules(&rule_table);
    let ast_entrypoints = ast_entrypoint_map(&rule_table);

    match check_file(
        &rule_table,
        &path_rules,
        &text_rules,
        &ast_entrypoints,
        path,
        file,
        settings,
        FixMode::Generate,
        settings::IgnoreAllowComments::Disabled,
    ) {
        Ok(violations) => {
            if violations.messages.is_empty() {
                return String::new();
            }

            let mut output = Vec::new();

            TextEmitter::default()
                .with_show_fix_status(true)
                .with_show_fix_diff(true)
                .with_show_source(true)
                .with_unsafe_fixes(crate::settings::UnsafeFixes::Enabled)
                .emit(&mut output, &violations.messages)
                .unwrap();

            String::from_utf8(output).unwrap()
        }
        Err(msg) => {
            panic!("Failed to process: {msg}");
        }
    }
}
