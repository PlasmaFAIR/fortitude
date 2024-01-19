use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    Check(CheckArgs),
    Explain(ExplainArgs),
}

/// Get descriptions, rationales, and solutions for each rule.
#[derive(Debug, clap::Parser)]
pub struct ExplainArgs {
    /// List of rules to explain. If omitted, explains all rules.
    #[arg()]
    pub rules: Vec<String>,
}

/// Perform static analysis on files and report issues.
#[derive(Debug, clap::Parser)]
pub struct CheckArgs {
    /// List of files to analyze
    #[arg(default_value = ".")]
    pub files: Vec<PathBuf>,
    /// Comma-separated list of extra rules to include.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
    /// Comma-separated list of rules to ignore.
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,
    /// Comma-separated list of the only rules you wish to use.
    #[arg(long, value_delimiter=',', conflicts_with_all=["include", "ignore"])]
    pub select: Vec<String>,
    /// Activate extra rules, strengthen others.
    #[arg(long, conflicts_with_all=["include", "ignore", "select"])]
    pub strict: bool,
    /// Set the maximum allowable line length.
    #[arg(long, default_value = "100")]
    pub line_length: usize,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
