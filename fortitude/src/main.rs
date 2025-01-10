use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use fortitude::check::check;
use fortitude::cli::{Cli, SubCommands};
use fortitude::explain::explain;
use fortitude::logging::set_up_logging;

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    set_up_logging(args.global_options.log_level())?;

    let status = match args.command {
        SubCommands::Check(check_args) => check(check_args, &args.global_options),
        SubCommands::Explain(args) => explain(args),
    };
    match status {
        Ok(code) => Ok(code),
        Err(_) => status,
    }
}
