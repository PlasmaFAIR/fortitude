//! ## The Fortitude Language Server

// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::{num::NonZeroUsize, process::ExitCode};

use anyhow::Context;
pub use edit::{PositionEncoding, TextDocument};
pub use server::Server;

#[macro_use]
mod message;

mod edit;
mod logging;
mod server;

pub(crate) const SERVER_NAME: &str = "fortitude";
pub(crate) const DIAGNOSTIC_NAME: &str = "Fortitude";

/// A common result type used in most cases where a
/// result type is needed.
pub(crate) type Result<T> = anyhow::Result<T>;

pub(crate) fn version() -> &'static str {
    fortitude_linter::VERSION
}

pub fn server() -> Result<ExitCode> {
    let four = NonZeroUsize::new(4).unwrap();

    // by default, we set the number of worker threads to `num_cpus`, with a maximum of 4.
    let worker_threads = std::thread::available_parallelism()
        .unwrap_or(four)
        .max(four);

    Server::new(worker_threads)
        .context("Failed to start server")?
        .run().map(|()| ExitCode::SUCCESS)
}
