use clap::{ArgAction::SetTrue, Parser, Subcommand};
use fortitude_workspace::configuration::{
    convert_file_and_extensions_to_include, resolve_bool_arg,
};
use path_absolutize::path_dedot;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use fortitude_linter::{
    fs::{FilePattern, GlobPath},
    logging::LogLevel,
    rule_selector::{RuleSelector, clap_completion::RuleSelectorParser, collect_per_file_ignores},
    settings::{
        FortranStandard, IgnoreAllowComments, OutputFormat, PatternPrefixPair, PreviewMode,
        ProgressBar, UnsafeFixes,
    },
};
use fortitude_workspace::configuration::{Configuration, ConfigurationTransformer};

use crate::commands::completions::config::{OptionString, OptionStringParser};

#[derive(Debug, Parser)]
#[command(
    author,
    name = "fortitude",
    about = "Fortitude: A Fortran linter, inspired by (and built upon) Ruff.",
    after_help = "For help with a specific command, see: `fortitude help <command>`."
)]
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
    #[clap(flatten)]
    log_level_args: LogLevelArgs,

    /// Path to a TOML configuration file
    #[arg(long, global = true, help_heading = "Global options")]
    pub config_file: Option<PathBuf>,
    /// Ignore all configuration files.
    #[arg(
        long,
        help_heading = "Global options",
        global = true,
        conflicts_with = "config_file"
    )]
    pub isolated: bool,
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
    Check(CheckCommand),
    Explain(ExplainCommand),
    /// Generate shell completion.
    #[clap(hide = true)]
    GenerateShellCompletion {
        shell: clap_complete_command::Shell,
    },
    /// Display Fortitude's version
    Version {
        #[arg(long, value_enum, default_value = "text")]
        output_format: HelpFormat,
    },
    /// List or describe the available configuration options.
    Config {
        /// Config key to show. Running the command with no key will
        /// show the top-level options. Nested options should be
        /// separated with '.', such as 'check.fix'.
        #[arg(
            value_parser = OptionStringParser,
            hide_possible_values = true
        )]
        option: Option<OptionString>,
        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        output_format: HelpFormat,
    },
    /// Run the language server
    Server(ServerCommand),
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum HelpFormat {
    Text,
    Json,
}

/// Perform static analysis on files and report issues.
#[derive(Debug, Parser, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CheckCommand {
    /// List of files or directories to check. Directories are searched recursively for
    /// Fortran files. The `--file-extensions` option can be used to control which files
    /// are included in the search.
    #[clap(help = "List of files or directories to check [default: .]")]
    pub files: Vec<PathBuf>,

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

    /// Set minimum Fortran standard to check files against.
    /// Options are "f2023", "f2018" (default), "f2008", "f2003", "f95"
    #[arg(long, value_enum)]
    pub target_std: Option<FortranStandard>,

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

/// CLI settings that are distinct from configuration (commands, lists of files,
/// etc.).
#[expect(clippy::struct_excessive_bools)]
pub struct CheckArguments {
    pub exit_non_zero_on_fix: bool,
    pub exit_zero: bool,
    pub files: Vec<PathBuf>,
    pub ignore_allow_comments: IgnoreAllowComments,
    pub output_file: Option<PathBuf>,
    pub show_files: bool,
    pub show_settings: bool,
    pub statistics: bool,
    pub stdin_filename: Option<PathBuf>,
}

/// Configuration-related arguments passed via the CLI.
#[derive(Default)]
pub struct ConfigArguments {
    // TODO: add other ruff bits like `isolated` and `overrides`
    /// Whether the user specified --isolated on the command line
    pub(crate) isolated: bool,
    /// The logging level to be used, derived from command-line arguments passed
    pub(crate) log_level: LogLevel,
    /// Path to a fpm.toml or fortitude.toml configuration file (etc.).
    /// Either 0 or 1 configuration file paths may be provided on the command line.
    config_file: Option<PathBuf>,
    /// Overrides provided via dedicated flags such as `--line-length` etc.
    /// These overrides take precedence over all configuration files,
    /// and also over all overrides specified using any `--config "KEY=VALUE"` flags.
    per_flag_overrides: ExplicitConfigOverrides,
}

impl ConfigArguments {
    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    fn from_cli_arguments(
        global_options: GlobalConfigArgs,
        per_flag_overrides: ExplicitConfigOverrides,
    ) -> anyhow::Result<Self> {
        let log_level = global_options.log_level();
        let config_file = global_options.config_file;
        let isolated = global_options.isolated;
        Ok(Self {
            isolated,
            log_level,
            config_file,
            per_flag_overrides,
        })
    }
}

impl ConfigurationTransformer for ConfigArguments {
    fn transform(&self, config: Configuration) -> Configuration {
        self.per_flag_overrides.transform(config)
    }
}

impl CheckCommand {
    /// Partition the CLI into command-line arguments and configuration
    /// overrides.
    pub fn partition(
        self,
        global_options: GlobalConfigArgs,
    ) -> anyhow::Result<(CheckArguments, ConfigArguments)> {
        let check_arguments = CheckArguments {
            exit_non_zero_on_fix: self.exit_non_zero_on_fix,
            exit_zero: self.exit_zero,
            files: self.files,
            ignore_allow_comments: self.ignore_allow_comments.into(),
            output_file: self.output_file,
            show_files: self.show_files,
            show_settings: self.show_settings,
            statistics: self.statistics,
            stdin_filename: self.stdin_filename,
        };

        let per_flag_overrides = ExplicitConfigOverrides {
            file_extensions: self.file_extensions,
            exclude: self.exclude,
            extend_exclude: self.extend_exclude,
            per_file_ignores: self.per_file_ignores,
            extend_per_file_ignores: self.extend_per_file_ignores,
            fix: resolve_bool_arg(self.fix, self.no_fix),
            fix_only: resolve_bool_arg(self.fix_only, self.no_fix_only),
            unsafe_fixes: resolve_bool_arg(self.unsafe_fixes, self.no_unsafe_fixes)
                .map(UnsafeFixes::from),
            show_fixes: resolve_bool_arg(self.show_fixes, self.no_show_fixes),
            force_exclude: resolve_bool_arg(self.force_exclude, self.no_force_exclude),
            line_length: self.line_length,
            respect_gitignore: resolve_bool_arg(self.respect_gitignore, self.no_respect_gitignore),
            preview: resolve_bool_arg(self.preview, self.no_preview).map(PreviewMode::from),
            output_format: self.output_format,
            select: self.select,
            ignore: self.ignore,
            extend_select: self.extend_select,
            target_std: self.target_std,
            progress_bar: self.progress_bar,
        };

        let config_args = ConfigArguments::from_cli_arguments(global_options, per_flag_overrides)?;
        Ok((check_arguments, config_args))
    }
}

/// Configuration overrides provided via dedicated CLI flags:
/// `--line-length`, `--respect-gitignore`, etc.
#[derive(Clone, Default)]
struct ExplicitConfigOverrides {
    file_extensions: Option<Vec<String>>,
    exclude: Option<Vec<FilePattern>>,
    extend_exclude: Option<Vec<FilePattern>>,
    per_file_ignores: Option<Vec<PatternPrefixPair>>,
    extend_per_file_ignores: Option<Vec<PatternPrefixPair>>,
    fix: Option<bool>,
    fix_only: Option<bool>,
    unsafe_fixes: Option<UnsafeFixes>,
    show_fixes: Option<bool>,
    force_exclude: Option<bool>,
    line_length: Option<usize>,
    respect_gitignore: Option<bool>,
    preview: Option<PreviewMode>,
    output_format: Option<OutputFormat>,
    select: Option<Vec<RuleSelector>>,
    extend_select: Option<Vec<RuleSelector>>,
    ignore: Option<Vec<RuleSelector>>,
    target_std: Option<FortranStandard>,
    progress_bar: Option<ProgressBar>,
}

impl ConfigurationTransformer for ExplicitConfigOverrides {
    fn transform(&self, mut config: Configuration) -> Configuration {
        if self.file_extensions.is_some() {
            config.include = convert_file_and_extensions_to_include(&None, &self.file_extensions)
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|pattern| {
                            let absolute = GlobPath::normalize(&pattern, path_dedot::CWD.as_path());
                            FilePattern::User(pattern, absolute)
                        })
                        .collect()
                })
        }
        if let Some(exclude) = &self.exclude {
            config.exclude = Some(exclude.clone());
        }
        if let Some(extend_exclude) = &self.extend_exclude {
            config.extend_exclude.extend(extend_exclude.clone());
        }
        if let Some(per_file_ignores) = &self.per_file_ignores {
            config.per_file_ignores = Some(collect_per_file_ignores(per_file_ignores.clone()));
        }
        if let Some(extend_per_file_ignores) = &self.extend_per_file_ignores {
            config
                .extend_per_file_ignores
                .extend(collect_per_file_ignores(extend_per_file_ignores.clone()));
        }
        if self.fix.is_some() {
            config.fix = self.fix;
        }
        if self.fix_only.is_some() {
            config.fix_only = self.fix_only;
        }
        if self.unsafe_fixes.is_some() {
            config.unsafe_fixes = self.unsafe_fixes;
        }
        if self.show_fixes.is_some() {
            config.show_fixes = self.show_fixes;
        }
        if self.force_exclude.is_some() {
            config.force_exclude = self.force_exclude;
        }
        if self.line_length.is_some() {
            config.line_length = self.line_length;
        }
        if self.respect_gitignore.is_some() {
            config.respect_gitignore = self.respect_gitignore;
        }
        if self.preview.is_some() {
            config.preview = self.preview;
        }
        if self.output_format.is_some() {
            config.output_format = self.output_format;
        }
        if let Some(select) = &self.select {
            config.select = Some(select.clone());
        }
        if let Some(extend_select) = &self.extend_select {
            config.extend_select.extend(extend_select.clone());
        }
        if let Some(ignore) = &self.ignore {
            config.ignore.extend(ignore.clone());
        }
        if self.target_std.is_some() {
            config.target_std = self.target_std;
        }
        if self.progress_bar.is_some() {
            config.progress_bar = self.progress_bar;
        }

        config
    }
}

/// Get descriptions, rationales, and solutions for each rule.
#[derive(Debug, clap::Parser, Clone)]
pub struct ExplainCommand {
    /// List of rules to explain. If omitted, explains all rules.
    #[arg(
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub rules: Vec<RuleSelector>,

    /// Show short summary of rule explanation
    #[arg(long)]
    pub summary: bool,

    /// Show available category names
    #[arg(long, conflicts_with = "rules", conflicts_with = "summary")]
    pub list_categories: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub output_format: HelpFormat,
}

#[derive(Copy, Clone, Debug, clap::Parser)]
pub struct ServerCommand {
    /// Enable preview mode. Use `--no-preview` to disable.
    ///
    /// This enables unstable server features and turns on the preview mode for the linter
    /// and the formatter.
    #[arg(long, overrides_with("no_preview"))]
    preview: bool,
    #[clap(long, overrides_with("preview"), hide = true)]
    no_preview: bool,
}

impl ServerCommand {
    pub(crate) fn resolve_preview(self) -> Option<bool> {
        resolve_bool_arg(Some(self.preview), Some(self.no_preview))
    }
}
