use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{rule_selector::RuleSelector, settings::OutputFormat, RuleSelectorParser};

/// Default extensions to check
pub const FORTRAN_EXTS: &[&str] = &[
    "f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23",
];

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: SubCommands,
    #[clap(flatten)]
    pub global_options: GlobalConfigArgs,
}

/// All configuration options that can be passed "globally",
/// i.e., can be passed to all subcommands
#[derive(Debug, Default, Clone, clap::Args)]
pub struct GlobalConfigArgs {
    /// Path to a TOML configuration file
    #[arg(long)]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
pub enum SubCommands {
    Check(CheckArgs),
    Explain(ExplainArgs),
}

/// Get descriptions, rationales, and solutions for each rule.
#[derive(Debug, clap::Parser, Clone, PartialEq)]
pub struct ExplainArgs {
    /// List of rules to explain. If omitted, explains all rules.
    #[arg(
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub rules: Vec<RuleSelector>,
}

/// Perform static analysis on files and report issues.
#[derive(Debug, clap::Parser, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CheckArgs {
    /// List of files to analyze
    #[arg(default_value = ".")]
    pub files: Option<Vec<PathBuf>>,
    /// Comma-separated list of rules to ignore.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub ignore: Option<Vec<RuleSelector>>,
    /// Comma-separated list of rule codes to enable (or ALL, to enable all rules).
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub select: Option<Vec<RuleSelector>>,
    /// Like --select, but adds additional rule codes on top of those already specified.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub extend_select: Option<Vec<RuleSelector>>,
    /// Set the maximum allowable line length.
    #[arg(long, default_value = "100")]
    pub line_length: Option<usize>,
    /// File extensions to check
    #[arg(long, value_delimiter = ',', default_values = FORTRAN_EXTS)]
    pub file_extensions: Option<Vec<String>>,

    /// Output serialization format for violations.
    /// The default serialization format is "full".
    #[arg(long, value_enum, env = "FORTITUDE_OUTPUT_FORMAT")]
    pub output_format: Option<OutputFormat>,
}
