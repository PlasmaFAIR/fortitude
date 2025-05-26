use clap::{Parser, Subcommand};
use std::path::PathBuf;

use fortitude_linter::cli::{CheckArgs, ExplainArgs};
use fortitude_linter::logging::LogLevel;

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
    /// Display Fortitude's version
    Version {
        #[arg(long, value_enum, default_value = "text")]
        output_format: HelpFormat,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum HelpFormat {
    Text,
    Json,
}
