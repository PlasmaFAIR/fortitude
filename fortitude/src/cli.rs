use clap::{ArgAction::SetTrue, Parser, Subcommand};
use serde::Deserialize;
use std::path::PathBuf;

use crate::{
    build,
    fs::FilePattern,
    logging::LogLevel,
    rule_selector::RuleSelector,
    settings::{OutputFormat, PatternPrefixPair, ProgressBar},
    RuleSelectorParser,
};

#[derive(Debug, Parser)]
#[command(
    author,
    name = "fortitude",
    about = "Fortitude: A Fortran linter, inspired by (and built upon) Ruff.",
    after_help = "For help with a specific command, see: `fortitude help <command>`."
)]
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
    #[clap(flatten)]
    log_level_args: LogLevelArgs,

    /// Path to a TOML configuration file
    #[arg(long)]
    pub config_file: Option<PathBuf>,
}

impl GlobalConfigArgs {
    pub fn log_level(&self) -> LogLevel {
        LogLevel::from(&self.log_level_args)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, clap::Args)]
pub struct LogLevelArgs {
    /// Enable verbose logging.
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub verbose: bool,
    /// Print diagnostics, but nothing else.
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub quiet: bool,
    /// Disable all logging (but still exit with status code "1" upon detecting diagnostics).
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub silent: bool,
}

impl From<&LogLevelArgs> for LogLevel {
    fn from(args: &LogLevelArgs) -> Self {
        if args.silent {
            Self::Silent
        } else if args.quiet {
            Self::Quiet
        } else if args.verbose {
            Self::Verbose
        } else {
            Self::Default
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum SubCommands {
    Check(CheckArgs),
    Explain(ExplainArgs),
    /// Generate shell completion.
    #[clap(hide = true)]
    GenerateShellCompletion {
        shell: clap_complete_command::Shell,
    },
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
#[derive(Debug, clap::Parser, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CheckArgs {
    /// List of files or directories to check. Directories are searched recursively for
    /// Fortran files. The `--file-extensions` option can be used to control which files
    /// are included in the search.
    #[arg(default_value = ".")]
    pub files: Option<Vec<PathBuf>>,

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

    /// Ignore any `allow` comments.
    #[arg(long)]
    pub ignore_allow_comments: bool,

    /// Output serialization format for violations.
    /// The default serialization format is "full".
    #[arg(long, value_enum, env = "FORTITUDE_OUTPUT_FORMAT")]
    pub output_format: Option<OutputFormat>,

    /// Specify file to write the linter output to (default: stdout).
    #[arg(short, long, env = "FORTITUDE_OUTPUT_FILE")]
    pub output_file: Option<PathBuf>,

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

    /// See the settings fortitude will use to check a given Fortran file.
    #[arg(long,
        // Fake subcommands.
        conflicts_with = "show_files",
        // conflicts_with = "show_settings",
    )]
    pub show_settings: bool,
    /// See the files fortitude will be run against with the current settings.
    #[arg(long,
          // Fake subcommands.
        // conflicts_with = "show_files",
        conflicts_with = "show_settings",
    )]
    pub show_files: bool,

    // Rule selection
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

    /// List of mappings from file pattern to code to exclude.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN:RULE_CODE",
        help_heading = "Rule selection"
    )]
    pub per_file_ignores: Option<Vec<PatternPrefixPair>>,

    /// Like `--per-file-ignores`, but adds additional ignores on top of those already specified.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN:RULE_CODE",
        help_heading = "Rule selection"
    )]
    pub extend_per_file_ignores: Option<Vec<PatternPrefixPair>>,

    // File selection
    /// File extensions to check
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "EXTENSION",
        help_heading = "File selection"
    )]
    pub file_extensions: Option<Vec<String>>,

    /// List of paths, used to omit files and/or directories from analysis.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN",
        help_heading = "File selection"
    )]
    pub exclude: Option<Vec<FilePattern>>,

    /// Like --exclude, but adds additional files and directories on top of those already excluded.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN",
        help_heading = "File selection"
    )]
    pub extend_exclude: Option<Vec<FilePattern>>,

    /// Enforce exclusions, even for paths passed to Fortitude directly on the command-line.
    /// Use `--no-force_exclude` to disable.
    #[arg(long, overrides_with("no_force_exclude"), help_heading = "File selection", action = SetTrue)]
    pub force_exclude: Option<bool>,
    #[clap(long, overrides_with("force_exclude"), hide = true, action = SetTrue)]
    pub no_force_exclude: Option<bool>,

    /// Respect `.gitignore`` files when determining which files to check.
    /// Use `--no-respect-gitignore` to disable.
    #[arg(long, overrides_with("no_respect_gitignore"), help_heading = "File selection", action = SetTrue)]
    pub respect_gitignore: Option<bool>,
    #[clap(long, overrides_with("respect_gitignore"), hide = true, action = SetTrue)]
    pub no_respect_gitignore: Option<bool>,

    // Options for individual rules
    /// Set the maximum allowable line length.
    #[arg(long, help_heading = "Per-Rule Options")]
    pub line_length: Option<usize>,

    // Miscellaneous
    /// The name of the file when passing it through stdin.
    #[arg(long, help_heading = "Miscellaneous")]
    pub stdin_filename: Option<PathBuf>,
    /// Exit with status code "0", even upon detecting lint violations.
    #[arg(
        short,
        long,
        help_heading = "Miscellaneous",
        conflicts_with = "exit_non_zero_on_fix",
        conflicts_with = "statistics"
    )]
    pub exit_zero: bool,
    /// Exit with a non-zero status code if any files were modified via fix, even if no lint violations remain.
    #[arg(
        long,
        help_heading = "Miscellaneous",
        conflicts_with = "exit_zero",
        conflicts_with = "statistics"
    )]
    pub exit_non_zero_on_fix: bool,
    /// Show counts for every rule with at least one violation.
    #[arg(long)]
    pub statistics: bool,
}
