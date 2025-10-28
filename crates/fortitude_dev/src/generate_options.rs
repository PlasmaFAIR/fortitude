// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

//! Generate a Markdown-compatible listing of configuration options for `fpm.toml`.
use anyhow::{Result, bail};
use itertools::Itertools;
use pretty_assertions::StrComparison;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use fortitude_workspace::options::Options;
use fortitude_workspace::options_base::{OptionField, OptionSet, OptionsMetadata, Visit};

use crate::ROOT_DIR;
use crate::generate_all::{Mode, REGENERATE_ALL_COMMAND};

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, default_value_t, value_enum)]
    pub(crate) mode: Mode,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let new = generate();

    if args.mode.is_dry_run() {
        print!("{new}");
        return Ok(());
    }

    let filename = "docs/settings.md";
    let file = PathBuf::from(ROOT_DIR).join(filename);
    let existing = fs::read_to_string(filename)?;

    match args.mode {
        Mode::Check => {
            if existing == new {
                println!("up-to-date: {filename}");
            } else {
                let comparison = StrComparison::new(&existing, &new);
                bail!("{filename} changed, please run `{REGENERATE_ALL_COMMAND}`:\n{comparison}");
            }
        }
        _ => {
            fs::write(file, new)?;
        }
    }
    Ok(())
}

pub(crate) fn generate() -> String {
    let mut output = String::new();

    output.push_str("# Settings\n\n");

    generate_set(
        &mut output,
        Set::Toplevel(Options::metadata()),
        &mut Vec::new(),
    );

    output
}

fn generate_set(output: &mut String, set: Set, parents: &mut Vec<Set>) {
    match &set {
        Set::Toplevel(_) => {
            output.push_str("## Top-level\n");
        }
        Set::Named { name, .. } => {
            let title = parents
                .iter()
                .filter_map(|set| set.name())
                .chain(std::iter::once(name.as_str()))
                .join(".");
            writeln!(output, "### `{title}`\n",).unwrap();
        }
    }

    if let Some(documentation) = set.metadata().documentation() {
        output.push_str(documentation);
        output.push('\n');
        output.push('\n');
    }

    let mut visitor = CollectOptionsVisitor::default();
    set.metadata().record(&mut visitor);

    let (mut fields, mut sets) = (visitor.fields, visitor.groups);

    fields.sort_unstable_by(|(name, _), (name2, _)| name.cmp(name2));
    sets.sort_unstable_by(|(name, _), (name2, _)| name.cmp(name2));

    parents.push(set);

    // Generate the fields.
    for (name, field) in &fields {
        emit_field(output, name, field, parents.as_slice());
        output.push_str("---\n\n");
    }

    // Generate all the sub-sets.
    for (set_name, sub_set) in &sets {
        generate_set(
            output,
            Set::Named {
                name: set_name.to_string(),
                set: *sub_set,
            },
            parents,
        );
    }

    parents.pop();
}

enum Set {
    Toplevel(OptionSet),
    Named { name: String, set: OptionSet },
}

impl Set {
    fn name(&self) -> Option<&str> {
        match self {
            Set::Toplevel(_) => None,
            Set::Named { name, .. } => Some(name),
        }
    }

    fn metadata(&self) -> &OptionSet {
        match self {
            Set::Toplevel(set) => set,
            Set::Named { set, .. } => set,
        }
    }
}

fn emit_field(output: &mut String, name: &str, field: &OptionField, parents: &[Set]) {
    let header_level = if parents.is_empty() { "###" } else { "####" };
    let parents_anchor = parents.iter().filter_map(|parent| parent.name()).join("_");

    if parents_anchor.is_empty() {
        output.push_str(&format!(
            "{header_level} [`{name}`](#{name}) {{: #{name} }}\n"
        ));
    } else {
        output.push_str(&format!(
            "{header_level} [`{name}`](#{parents_anchor}_{name}) {{: #{parents_anchor}_{name} }}\n"
        ));

        // the anchor used to just be the name, but now it's the group name
        // for backwards compatibility, we need to keep the old anchor
        output.push_str(&format!("<span id=\"{name}\"></span>\n"));
    }

    output.push('\n');

    if let Some(deprecated) = &field.deprecated {
        output.push_str("!!! warning \"Deprecated\"\n");
        output.push_str("    This option has been deprecated");

        if let Some(since) = deprecated.since {
            write!(output, " in {since}").unwrap();
        }

        output.push('.');

        if let Some(message) = deprecated.message {
            writeln!(output, " {message}").unwrap();
        }

        output.push('\n');
    }

    output.push_str(field.doc);
    output.push_str("\n\n");
    output.push_str(&format!("**Default value**: `{}`\n", field.default));
    output.push('\n');
    output.push_str(&format!("**Type**: `{}`\n", field.value_type));
    output.push('\n');
    output.push_str("**Example usage**:\n\n");
    output.push_str(&format_tab(
        "`fpm.toml`",
        &format_header(field.scope, parents, ConfigurationFile::FpmToml),
        field.example,
    ));
    output.push_str(&format_tab(
        "`fortitude.toml` or `.fortitude.toml`",
        &format_header(field.scope, parents, ConfigurationFile::FortitudeToml),
        field.example,
    ));
    output.push('\n');
}

fn format_tab(tab_name: &str, header: &str, content: &str) -> String {
    format!(
        "=== \"{}\"\n\n    ```toml\n    {}\n{}\n    ```\n",
        tab_name,
        header,
        textwrap::indent(content, "    ")
    )
}

/// Format the TOML header for the example usage for a given option.
///
/// For example: `[extra.fortitude.check]` in `fpm.toml` or `[check]` in
/// `fortitude.toml`.
fn format_header(scope: Option<&str>, parents: &[Set], configuration: ConfigurationFile) -> String {
    let tool_parent = match configuration {
        ConfigurationFile::FpmToml => Some("extra.fortitude"),
        ConfigurationFile::FortitudeToml => None,
    };

    let header = tool_parent
        .into_iter()
        .chain(parents.iter().filter_map(|parent| parent.name()))
        .chain(scope)
        .join(".");

    if header.is_empty() {
        String::new()
    } else {
        format!("[{header}]")
    }
}

#[derive(Debug, Copy, Clone)]
enum ConfigurationFile {
    FpmToml,
    FortitudeToml,
}

#[derive(Default)]
struct CollectOptionsVisitor {
    groups: Vec<(String, OptionSet)>,
    fields: Vec<(String, OptionField)>,
}

impl Visit for CollectOptionsVisitor {
    fn record_set(&mut self, name: &str, group: OptionSet) {
        self.groups.push((name.to_owned(), group));
    }

    fn record_field(&mut self, name: &str, field: OptionField) {
        self.fields.push((name.to_owned(), field));
    }
}
