use std::process::ExitCode;

use fortitude::check::check;
use fortitude::cli::{parse_args, SubCommands};
use fortitude::explain::explain;

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(_) => return ExitCode::FAILURE,
    };
    let status = match args.command {
        SubCommands::Check(args) => check(args),
        SubCommands::Explain(args) => explain(args),
    };
    match status {
        Ok(code) => code,
        Err(_) => ExitCode::FAILURE,
    }
}
