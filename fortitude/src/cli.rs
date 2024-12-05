use clap::{ArgAction::SetTrue, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    build, rule_selector::RuleSelector, settings::OutputFormat, settings::ProgressBar,
    RuleSelectorParser,
};

/// Default extensions to check
pub const FORTRAN_EXTS: &[&str] = &[
    "f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23",
];

#[derive(Debug, Parser)]
#[command(version = build::CLAP_LONG_VERSION, about)]
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
    /// List of files or directories to check. Directories are searched recursively for
    /// Fortran files. The `--file-extensions` option can be used to control which files
    /// are included in the search.
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

    /// Apply fixes to resolve lint violations.
    /// Use `--no-fix` to disable or `--unsafe-fixes` to include unsafe fixes.
    #[arg(long, overrides_with("no_fix"), action = clap::ArgAction::SetTrue)]
    pub fix: Option<bool>,
    #[clap(long, overrides_with("fix"), hide = true, action = SetTrue)]
    pub no_fix: Option<bool>,
    /// Include fixes that may not retain the original intent of the code.
    /// Use `--no-unsafe-fixes` to disable.
    #[arg(long, overrides_with("no_unsafe_fixes"), action = SetTrue)]
    pub unsafe_fixes: Option<bool>,
    #[arg(long, overrides_with("unsafe_fixes"), hide = true, action = SetTrue)]
    pub no_unsafe_fixes: Option<bool>,
    /// Show an enumeration of all fixed lint violations.
    /// Use `--no-show-fixes` to disable.
    #[arg(long, overrides_with("no_show_fixes"), action = SetTrue)]
    pub show_fixes: Option<bool>,
    #[clap(long, overrides_with("show_fixes"), hide = true, action = SetTrue)]
    pub no_show_fixes: Option<bool>,
    /// Apply fixes to resolve lint violations, but don't report on, or exit non-zero for, leftover violations. Implies `--fix`.
    /// Use `--no-fix-only` to disable or `--unsafe-fixes` to include unsafe fixes.
    #[arg(long, overrides_with("no_fix_only"), action = SetTrue)]
    pub fix_only: Option<bool>,
    #[clap(long, overrides_with("fix_only"), hide = true, action = SetTrue)]
    pub no_fix_only: Option<bool>,

    /// Output serialization format for violations.
    /// The default serialization format is "full".
    #[arg(long, value_enum, env = "FORTITUDE_OUTPUT_FORMAT")]
    pub output_format: Option<OutputFormat>,

    /// Enable preview mode; checks will include unstable rules and fixes.
    /// Use `--no-preview` to disable.
    #[arg(long, overrides_with("no_preview"), action = SetTrue)]
    pub preview: Option<bool>,
    #[clap(long, overrides_with("preview"), hide = true, action = SetTrue)]
    pub no_preview: Option<bool>,

    /// Progress bar settings.
    /// Options are "off" (default), "ascii", and "fancy"
    #[arg(long, value_enum)]
    pub progress_bar: Option<ProgressBar>,
}
