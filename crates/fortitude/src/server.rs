use std::process::ExitCode;

use crate::cli::ServerCommand;
use anyhow::Result;

pub fn server_command(args: ServerCommand) -> Result<ExitCode> {
    fortitude_server::server(args.resolve_preview())?;
    Ok(ExitCode::SUCCESS)
}
