// Adapted from ruff
// Copyright 2022-2026 Charles Marsh
// SPDX-License-Identifier: MIT

use std::any::Any;
use std::backtrace::BacktraceStatus;
use std::cell::Cell;
use std::panic::Location;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct PanicError {
    pub location: Option<String>,
    pub payload: Payload,
    pub backtrace: Option<std::backtrace::Backtrace>,
}

#[derive(Debug)]
pub struct Payload(Box<dyn std::any::Any + Send>);

impl Payload {
    pub fn downcast_ref<R: Any>(&self) -> Option<&R> {
        self.0.downcast_ref::<R>()
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.0.downcast_ref::<String>() {
            f.write_str(s)
        } else if let Some(s) = self.0.downcast_ref::<&str>() {
            f.write_str(s)
        } else {
            f.write_str("Box<dyn Any>")
        }
    }
}

impl PanicError {
    pub fn resume_unwind(self) -> ! {
        std::panic::resume_unwind(self.payload.0)
    }

    pub fn to_diagnostic_message(&self, path: Option<impl std::fmt::Display>) -> String {
        use std::fmt::Write;

        let mut message = String::new();
        message.push_str("Panicked");

        if let Some(location) = &self.location {
            let _ = write!(&mut message, " at {location}");
        }

        if let Some(path) = path {
            let _ = write!(&mut message, " when checking `{path}`");
        }

        let _ = write!(&mut message, ": `{payload}`", payload = self.payload);

        message
    }
}

impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "panicked at")?;
        if let Some(location) = &self.location {
            write!(f, " {location}")?;
        }

        write!(f, ":\n{payload}", payload = self.payload)?;

        if let Some(backtrace) = &self.backtrace {
            match backtrace.status() {
                BacktraceStatus::Disabled => {
                    writeln!(
                        f,
                        "\nrun with `RUST_BACKTRACE=1` environment variable to display a backtrace"
                    )?;
                }
                BacktraceStatus::Captured => {
                    writeln!(f, "\nBacktrace: {backtrace}")?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct CapturedPanicInfo {
    backtrace: Option<std::backtrace::Backtrace>,
    location: Option<String>,
}

thread_local! {
    static CAPTURE_PANIC_INFO: Cell<bool> = const { Cell::new(false) };
    static LAST_BACKTRACE: Cell<CapturedPanicInfo> = const {
        Cell::new(CapturedPanicInfo { backtrace: None, location: None })
    };
}

fn install_hook() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let should_capture = CAPTURE_PANIC_INFO.with(Cell::get);
            if !should_capture {
                return (*prev)(info);
            }

            let location = info.location().map(Location::to_string);
            let backtrace = Some(std::backtrace::Backtrace::capture());

            LAST_BACKTRACE.set(CapturedPanicInfo {
                backtrace,
                location,
            });
        }));
    });
}

/// Invokes a closure, capturing and returning the cause of an unwinding panic if one occurs.
///
/// ### Thread safety
///
/// This is implemented by installing a custom [panic hook](std::panic::set_hook).  This panic hook
/// is a global resource.  The hook that we install captures panic info in a thread-safe manner,
/// and also ensures that any threads that are _not_ currently using this `catch_unwind` wrapper
/// still use the previous hook (typically the default hook, which prints out panic information to
/// stderr).
///
/// We assume that there is nothing else running in this process that needs to install a competing
/// panic hook. We are careful to install our custom hook only once, and we do not ever restore
/// the previous hook (since you can always retain the previous hook's behavior by not calling this
/// wrapper).
pub fn catch_unwind<F, R>(f: F) -> Result<R, PanicError>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    install_hook();
    let prev_should_capture = CAPTURE_PANIC_INFO.replace(true);
    let result = std::panic::catch_unwind(f).map_err(|payload| {
        // Try to get the backtrace and location from our custom panic hook.
        // The custom panic hook only runs once when `panic!` is called (or similar). It doesn't
        // run when the panic is propagated with `std::panic::resume_unwind`. The panic hook
        // is also not called when the panic is raised with `std::panic::resume_unwind`.
        // Because of that, always take the payload from `catch_unwind` because it may have been transformed
        // by an inner `std::panic::catch_unwind` handlers and only use the information
        // from the custom handler to enrich the error with the backtrace and location.
        let CapturedPanicInfo {
            location,
            backtrace,
        } = LAST_BACKTRACE.with(Cell::take);

        PanicError {
            location,
            payload: Payload(payload),
            backtrace,
        }
    });
    CAPTURE_PANIC_INFO.set(prev_should_capture);
    result
}
