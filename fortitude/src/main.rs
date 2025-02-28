use std::io::{stdout, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
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
        SubCommands::GenerateShellCompletion { shell } => {
            shell.generate(&mut Cli::command(), &mut stdout());
            return Ok(ExitCode::SUCCESS);
        }
    };
    match status {
        Ok(code) => Ok(code),
        Err(err) => {
            {
                // Exit "gracefully" on broken pipe errors.
                //
                // See: https://github.com/BurntSushi/ripgrep/blob/bf63fe8f258afc09bae6caa48f0ae35eaf115005/crates/core/main.rs#L47C1-L61C14
                for cause in err.chain() {
                    if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
                        if ioerr.kind() == std::io::ErrorKind::BrokenPipe {
                            return Ok(ExitCode::from(0));
                        }
                    }
                }

                // Use `writeln` instead of `eprintln` to avoid panicking when the stderr pipe is broken.
                let mut stderr = std::io::stderr().lock();

                // This communicates that this isn't a linter error but ruff itself hard-errored for
                // some reason (e.g. failed to resolve the configuration)
                writeln!(stderr, "{}", "fortitude failed".red().bold()).ok();

                // TODO: handle reporting multiple/chain of errors. Currently
                // only most recent error is reported
            }
            Err(err)
        }
    }
}
