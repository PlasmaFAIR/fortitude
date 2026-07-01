use anyhow::bail;
use clap::{
    ArgAction::SetTrue,
    Parser, Subcommand,
    builder::{TypedValueParser, ValueParserFactory},
};
use fortitude_workspace::{
    configuration::{convert_file_and_extensions_to_include, resolve_bool_arg},
    options::Options,
};
use itertools::Itertools;
use path_absolutize::path_dedot;
use ruff_options_metadata::{OptionEntry, OptionsMetadata};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt::Write as _, ops::Deref};

use fortitude_linter::{
    fs::{FilePattern, GlobPath},
    line_filter::{Filter, FilterMap},
    logging::LogLevel,
    rule_selector::{RuleSelector, clap_completion::RuleSelectorParser, collect_per_file_ignores},
    settings::{
        FortranStandard, IgnoreAllowComments, OutputFormat, PatternPrefixPair, PreviewMode,
        ProgressBar, UnsafeFixes,
    },
    warn_user_once_by_message,
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

    /// Either a path to a TOML configuration file
    /// (`[fpm,fortitude,pyproject].toml`), or a TOML `<KEY> = <VALUE>` pair
    /// (such as you might find in a `fortitude.toml` configuration file)
    /// overriding a specific configuration option.  Overrides of individual
    /// settings using this option always take precedence over all configuration
    /// files, including configuration files that were also specified using
    /// `--config`.
    #[arg(
        long,
        action = clap::ArgAction::Append,
        value_name = "CONFIG_OPTION",
        value_parser = ConfigArgumentParser,
        global = true,
        help_heading = "Global options",
    )]
    pub config: Vec<SingleConfigArgument>,
    /// Path to a TOML configuration file. (Deprecated: Use `--config=<filename>` instead.)
    #[arg(long, global = true, help_heading = "Global options", hide = true)]
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

    /// Avoid writing any fixed files back; instead, output a diff for each changed file to stdout, and exit 0 if there are no diffs.
    /// Implies `--fix-only`.
    #[arg(long, conflicts_with = "show_fixes")]
    pub diff: bool,

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

    /// List of rule codes to treat as eligible for fix. Only has an effect if `--fix` is also enabled.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub fixable: Option<Vec<RuleSelector>>,

    /// List of rule codes to treat as ineligible for fix. Only has an effect if `--fix` is also enabled.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub unfixable: Option<Vec<RuleSelector>>,

    /// Like `--fixable`, but adds additional rule codes on top of those already specified.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "RULE_CODE",
        value_parser = RuleSelectorParser,
        help_heading = "Rule selection",
        hide_possible_values = true
    )]
    pub extend_fixable: Option<Vec<RuleSelector>>,

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

    /// List of files with line ranges to filter warnings. The format is JSON
    /// array of objects:
    ///  [
    ///    {"name": "file1.f90", "lines": [[6, 7], [42, 45]]},
    ///    {"name": "file2.f90"}
    ///  ]
    /// Line ranges include the end.
    #[arg(
        long,
        help_heading = "File selection",
        value_name = "LINE_FILTER",
        conflicts_with = "git_staged",
        conflicts_with = "git_since"
    )]
    pub line_filter: Option<Filter>,

    /// Only run on files that have been staged in a git repository
    #[arg(
        long,
        help_heading = "File selection",
        conflicts_with = "line_filter",
        conflicts_with = "git_since"
    )]
    pub git_staged: bool,

    /// Only run on files that differ between the files in the working directory
    /// of a git repository and `COMMIT`. `COMMIT` can be most things that look
    /// like a commit, for example `main`, `0f3abc`, `HEAD~`
    #[arg(
        long,
        help_heading = "File selection",
        value_name = "COMMIT",
        conflicts_with = "line_filter",
        conflicts_with = "git_staged"
    )]
    pub git_since: Option<String>,

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
    pub diff: bool,
    pub exit_non_zero_on_fix: bool,
    pub exit_zero: bool,
    pub files: Vec<PathBuf>,
    pub git_staged: bool,
    pub git_since: Option<String>,
    pub ignore_allow_comments: IgnoreAllowComments,
    pub line_filter: Option<FilterMap>,
    pub output_file: Option<PathBuf>,
    pub show_files: bool,
    pub show_settings: bool,
    pub statistics: bool,
    pub stdin_filename: Option<PathBuf>,
}

/// Configuration-related arguments passed via the CLI.
#[derive(Default)]
pub struct ConfigArguments {
    /// Whether the user specified --isolated on the command line
    pub(crate) isolated: bool,
    /// The logging level to be used, derived from command-line arguments passed
    pub(crate) log_level: LogLevel,
    /// Path to a fpm.toml or fortitude.toml configuration file (etc.).
    /// Either 0 or 1 configuration file paths may be provided on the command line.
    config_file: Option<PathBuf>,
    /// Overrides provided via the `--config "KEY=VALUE"` option.
    /// An arbitrary number of these overrides may be provided on the command line.
    /// These overrides take precedence over all configuration files,
    /// even configuration files that were also specified using `--config`.
    overrides: Configuration,
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
        let deprecated_config_file = global_options.config_file;
        let config_options = global_options.config;
        let isolated = global_options.isolated;

        if deprecated_config_file.is_some() {
            warn_user_once_by_message!(
                "The `--config-file` option is now deprecated in favour of `--config`"
            );
        }

        let mut config_file: Option<PathBuf> = deprecated_config_file;
        let mut overrides = Configuration::default();

        for option in config_options {
            match option {
                SingleConfigArgument::SettingsOverride(overridden_option) => {
                    let overridden_option = Arc::try_unwrap(overridden_option)
                        .unwrap_or_else(|option| option.deref().clone());
                    overrides = overrides.combine(Configuration::from_options(
                        overridden_option,
                        &path_dedot::CWD,
                    ));
                }
                SingleConfigArgument::FilePath(path) => {
                    if isolated {
                        bail!(
                            "\
The argument `--config={}` cannot be used with `--isolated`

  tip: You cannot specify a configuration file and also specify `--isolated`,
       as `--isolated` causes fortitude to ignore all configuration files.
       For more information, try `--help`.
",
                            path.display()
                        );
                    }
                    if let Some(ref config_file) = config_file {
                        let (first, second) = (config_file.display(), path.display());
                        bail!(
                            "\
You cannot specify more than one configuration file on the command line.

  tip: remove either `--config={first}` or `--config={second}`.
       For more information, try `--help`.
"
                        );
                    }
                    config_file = Some(path);
                }
            }
        }

        Ok(Self {
            isolated,
            log_level,
            config_file,
            overrides,
            per_flag_overrides,
        })
    }
}

impl ConfigurationTransformer for ConfigArguments {
    fn transform(&self, config: Configuration) -> Configuration {
        let with_config_overrides = self.overrides.clone().combine(config);
        self.per_flag_overrides.transform(with_config_overrides)
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
            diff: self.diff,
            exit_non_zero_on_fix: self.exit_non_zero_on_fix,
            exit_zero: self.exit_zero,
            files: self.files,
            git_staged: self.git_staged,
            git_since: self.git_since,
            line_filter: self.line_filter.map(FilterMap::new),
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
            fixable: self.fixable,
            unfixable: self.unfixable,
            extend_fixable: self.extend_fixable,
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
    fixable: Option<Vec<RuleSelector>>,
    unfixable: Option<Vec<RuleSelector>>,
    extend_fixable: Option<Vec<RuleSelector>>,
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
        if let Some(fixable) = &self.fixable {
            config.fixable = Some(fixable.clone());
        }
        if let Some(unfixable) = &self.unfixable {
            config.unfixable.extend(unfixable.clone());
        }
        if let Some(extend_fixable) = &self.extend_fixable {
            config.extend_fixable.extend(extend_fixable.clone());
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

/// Enumeration of various ways in which a --config CLI flag
/// could be invalid
#[derive(Debug)]
enum InvalidConfigFlagReason {
    InvalidToml(toml::de::Error),
    /// It was valid TOML, but not a valid fortitude config file.
    /// E.g. the user tried to select a rule that doesn't exist,
    /// or tried to enable a setting that doesn't exist
    ValidTomlButInvalidFortitudeSchema(toml::de::Error),
    /// It was a valid fortitude config file, but the user tried to pass a
    /// value for `extend` as part of the config override.
    /// `extend` is special, because it affects which config files we look at
    /// in the first place. We currently only parse --config overrides *after*
    /// we've combined them with all the arguments from the various config files
    /// that we found, so trying to override `extend` as part of a --config
    /// override is forbidden.
    // TODO(peter): use this when we get `extend`
    #[allow(dead_code)]
    ExtendPassedViaConfigFlag,
}

impl InvalidConfigFlagReason {
    const fn description(&self) -> &'static str {
        match self {
            Self::InvalidToml(_) => "The supplied argument is not valid TOML",
            Self::ValidTomlButInvalidFortitudeSchema(_) => {
                "Could not parse the supplied argument as a `fortitude.toml` configuration option"
            }
            Self::ExtendPassedViaConfigFlag => "Cannot include `extend` in a --config flag value",
        }
    }
}

/// Enumeration to represent a single `--config` argument
/// passed via the CLI.
///
/// Using the `--config` flag, users may pass 0 or 1 paths
/// to configuration files and an arbitrary number of
/// "inline TOML" overrides for specific settings.
///
/// For example:
///
/// ```sh
/// fortitude check --config "path/to/fortitude.toml" --config "extend-select=['E501', 'F841']" --config "lint.per-file-ignores = {'some_file.py' = ['F841']}"
/// ```
#[derive(Clone, Debug)]
pub enum SingleConfigArgument {
    FilePath(PathBuf),
    SettingsOverride(Arc<Options>),
}

#[derive(Clone)]
pub struct ConfigArgumentParser;

impl ValueParserFactory for SingleConfigArgument {
    type Parser = ConfigArgumentParser;

    fn value_parser() -> Self::Parser {
        ConfigArgumentParser
    }
}

impl TypedValueParser for ConfigArgumentParser {
    type Value = SingleConfigArgument;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        // Convert to UTF-8.
        let Some(value) = value.to_str() else {
            // But respect non-UTF-8 paths.
            let path_to_config_file = PathBuf::from(value);
            if path_to_config_file.is_file() {
                return Ok(SingleConfigArgument::FilePath(path_to_config_file));
            }
            return Err(clap::Error::new(clap::error::ErrorKind::InvalidUtf8));
        };

        // Expand environment variables and tildes.
        if let Ok(path_to_config_file) =
            shellexpand::full(value).map(|config| PathBuf::from(&*config))
            && path_to_config_file.is_file()
        {
            return Ok(SingleConfigArgument::FilePath(path_to_config_file));
        }

        let config_parse_error = match toml::Table::from_str(value) {
            Ok(table) => match table.try_into::<Options>() {
                Ok(option) => {
                    return Ok(SingleConfigArgument::SettingsOverride(Arc::new(option)));
                }
                Err(underlying_error) => {
                    InvalidConfigFlagReason::ValidTomlButInvalidFortitudeSchema(underlying_error)
                }
            },
            Err(underlying_error) => InvalidConfigFlagReason::InvalidToml(underlying_error),
        };

        let mut new_error = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
            new_error.insert(
                clap::error::ContextKind::InvalidArg,
                clap::error::ContextValue::String(arg.to_string()),
            );
        }
        new_error.insert(
            clap::error::ContextKind::InvalidValue,
            clap::error::ContextValue::String(value.to_string()),
        );

        let underlying_error = match &config_parse_error {
            InvalidConfigFlagReason::ExtendPassedViaConfigFlag => {
                let tip = config_parse_error.description().into();
                new_error.insert(
                    clap::error::ContextKind::Suggested,
                    clap::error::ContextValue::StyledStrs(vec![tip]),
                );
                return Err(new_error);
            }
            InvalidConfigFlagReason::InvalidToml(underlying_error)
            | InvalidConfigFlagReason::ValidTomlButInvalidFortitudeSchema(underlying_error) => {
                underlying_error
            }
        };

        // small hack so that multiline tips
        // have the same indent on the left-hand side:
        let tip_indent = " ".repeat("  tip: ".len());

        let mut tip = format!(
            "\
A `--config` flag must either be a path to a `.toml` configuration file
{tip_indent}or a TOML `<KEY> = <VALUE>` pair overriding a specific configuration
{tip_indent}option"
        );

        // Here we do some heuristics to try to figure out whether
        // the user was trying to pass in a path to a configuration file
        // or some inline TOML.
        // We want to display the most helpful error to the user as possible.
        if Path::new(value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        {
            if !value.contains('=') {
                let _ = write!(
                    &mut tip,
                    "

It looks like you were trying to pass a path to a configuration file.
The path `{value}` does not point to a configuration file"
                );
            }
        } else if let Some((key, value)) = value.split_once('=') {
            let key = key.trim_ascii();
            let value = value.trim_ascii_start();

            match Options::metadata().find(key) {
                Some(OptionEntry::Set(set)) if !value.starts_with('{') => {
                    let prefixed_subfields = set
                        .collect_fields()
                        .iter()
                        .map(|(name, _)| format!("- `{key}.{name}`"))
                        .join("\n");

                    let _ = write!(
                        &mut tip,
                        "

`{key}` is a table of configuration options.
Did you want to override one of the table's subkeys?

Possible choices:

{prefixed_subfields}"
                    );
                }
                _ => {
                    let _ = write!(
                        &mut tip,
                        "\n\n{}:\n\n{underlying_error}",
                        config_parse_error.description()
                    );
                }
            }
        }
        let tip = tip.trim_end().to_owned().into();

        new_error.insert(
            clap::error::ContextKind::Suggested,
            clap::error::ContextValue::StyledStrs(vec![tip]),
        );

        Err(new_error)
    }
}
