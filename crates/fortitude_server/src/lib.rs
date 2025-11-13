//! ## The Fortitude Language Server

// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::num::NonZeroUsize;

use anyhow::Context as _;
pub use edit::{DocumentKey, PositionEncoding, TextDocument};
use lsp_types::CodeActionKind;
pub use server::{ConnectionSender, MainLoopSender, Server};
pub use session::{Client, ClientOptions, DocumentQuery, DocumentSnapshot, GlobalOptions, Session};
pub use workspace::{Workspace, Workspaces};

use crate::server::ConnectionInitializer;

mod edit;
mod fix;
mod lint;
mod logging;
mod resolve;
mod server;
mod session;
mod workspace;

pub(crate) const SERVER_NAME: &str = "fortitude";
pub(crate) const DIAGNOSTIC_NAME: &str = "Fortitude";

pub const SOURCE_FIX_ALL_FORTITUDE: CodeActionKind = CodeActionKind::new("source.fixAll.fortitude");

/// A common result type used in most cases where a
/// result type is needed.
pub(crate) type Result<T> = anyhow::Result<T>;

pub(crate) fn version() -> &'static str {
    fortitude_linter::VERSION
}

pub fn server(preview: Option<bool>) -> Result<()> {
    let four = NonZeroUsize::new(4).unwrap();

    // by default, we set the number of worker threads to `num_cpus`, with a maximum of 4.
    let worker_threads = std::thread::available_parallelism()
        .unwrap_or(four)
        .max(four);

    let (connection, io_threads) = ConnectionInitializer::stdio();

    let server_result = Server::new(worker_threads, connection, preview)
        .context("Failed to start server")?
        .run();

    let io_result = io_threads.join();

    let result = match (server_result, io_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(server), Err(io)) => Err(server).context(format!("IO thread error: {io}")),
        (Err(server), _) => Err(server),
        (_, Err(io)) => Err(io).context("IO thread error"),
    };

    if let Err(err) = result.as_ref() {
        tracing::warn!("Server shut down with an error: {err}");
    } else {
        tracing::info!("Server shut down");
    }

    result
}
