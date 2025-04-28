//! The logging system for `ruff server`.
//!
//! Log messages are controlled by the `logLevel` setting which defaults to `"info"`. Log messages
//! are written to `stderr` by default, which should appear in the logs for most LSP clients. A
//! `logFile` path can also be specified in the settings, and output will be directed there
//! instead.
use core::str;
use serde::Deserialize;

/// The log level for the server as provided by the client during initialization.
///
/// The default log level is `info`.
#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}
