//! Scheduling, I/O, and API endpoints.

use lsp_server::Connection;
use lsp_types::InitializeParams;
use lsp_types as types;
use std::num::NonZeroUsize;
// The new PanicInfoHook name requires MSRV >= 1.82
#[allow(deprecated)]
use std::panic::PanicInfo;
use std::sync::Arc;
use types::ClientCapabilities;
use types::DiagnosticOptions;
use types::WorkDoneProgressOptions;

pub(crate) use self::connection::ConnectionInitializer;
pub use self::connection::ConnectionSender;
use self::schedule::spawn_main_loop;
use crate::session::AllOptions;
use crate::workspace::Workspaces;
use crate::Client;
use crate::PositionEncoding;
pub use crate::server::main_loop::MainLoopSender;
pub(crate) use crate::server::main_loop::{Event, MainLoopReceiver};
use crate::session::Session;
pub(crate) use api::Error;

mod api;
mod connection;
mod main_loop;
mod schedule;

pub(crate) type Result<T> = std::result::Result<T, api::Error>;

pub struct Server {
    connection: Connection,
    client_capabilities: ClientCapabilities,
    main_loop_receiver: MainLoopReceiver,
    main_loop_sender: MainLoopSender,
    worker_threads: NonZeroUsize,
    session: Session,
}

impl Server {
    pub fn new(
        worker_threads: NonZeroUsize,
        connection: ConnectionInitializer,
    ) -> crate::Result<Self> {
        let (id, init_params) = connection.initialize_start()?;

        let client_capabilities = init_params.capabilities;
        let position_encoding = Self::find_best_position_encoding(&client_capabilities);
        let server_capabilities = Self::server_capabilities(position_encoding);

        let connection = connection.initialize_finish(
            id,
            &server_capabilities,
            crate::SERVER_NAME,
            crate::version(),
        )?;

        let (main_loop_sender, main_loop_receiver) = crossbeam::channel::bounded(32);

        let InitializeParams {
            initialization_options,
            workspace_folders,
            ..
        } = init_params;

        let client = Client::new(main_loop_sender.clone(), connection.sender.clone());
        let all_options = AllOptions::from_value(
            initialization_options
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::default())),
            &client,
        );

        let AllOptions {
            global: global_options,
            workspace: workspace_options,
        } = all_options;

        crate::logging::init_logging(
            global_options.tracing.log_level.unwrap_or_default(),
            global_options.tracing.log_file.as_deref(),
        );

        let workspaces = Workspaces::from_workspace_folders(
            workspace_folders,
            workspace_options.unwrap_or_default(),
        )?;

        let global = global_options.into_settings(client.clone());

        Ok(Self {
            connection,
            worker_threads,
            main_loop_sender,
            main_loop_receiver,
            session: Session::new(
                &client_capabilities,
                position_encoding,
                global,
                &workspaces,
                &client,
            )?,
            client_capabilities,
        })
    }

    pub fn run(mut self) -> crate::Result<()> {
        let client = Client::new(
            self.main_loop_sender.clone(),
            self.connection.sender.clone(),
        );

        let _panic_hook = ServerPanicHookHandler::new(client);

        spawn_main_loop(move || self.main_loop())?.join()
    }

    fn find_best_position_encoding(client_capabilities: &ClientCapabilities) -> PositionEncoding {
        client_capabilities
            .general
            .as_ref()
            .and_then(|general_capabilities| general_capabilities.position_encodings.as_ref())
            .and_then(|encodings| {
                encodings
                    .iter()
                    .filter_map(|encoding| PositionEncoding::try_from(encoding).ok())
                    .max() // this selects the highest priority position encoding
            })
            .unwrap_or_default()
    }

    fn server_capabilities(position_encoding: PositionEncoding) -> types::ServerCapabilities {
        types::ServerCapabilities {
            position_encoding: Some(position_encoding.into()),
            code_action_provider: None,
            workspace: None,
            diagnostic_provider: Some(types::DiagnosticServerCapabilities::Options(
                DiagnosticOptions {
                    identifier: Some(crate::DIAGNOSTIC_NAME.into()),
                    // multi-file analysis could change this
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                },
            )),
            execute_command_provider: None,
            hover_provider: None,
            text_document_sync: None,
            ..Default::default()
        }
    }
}

// The new PanicInfoHook name requires MSRV >= 1.82
#[allow(deprecated)]
type PanicHook = Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send>;
struct ServerPanicHookHandler {
    hook: Option<PanicHook>,
    // Hold on to the strong reference for as long as the panic hook is set.
    _client: Arc<Client>,
}

impl ServerPanicHookHandler {
    fn new(client: Client) -> Self {
        let hook = std::panic::take_hook();
        let client = Arc::new(client);

        // Use a weak reference to the client because it must be dropped when exiting or the
        // io-threads join hangs forever (because client has a reference to the connection sender).
        let hook_client = Arc::downgrade(&client);

        // When we panic, try to notify the client.
        std::panic::set_hook(Box::new(move |panic_info| {
            use std::io::Write;

            let backtrace = std::backtrace::Backtrace::force_capture();
            tracing::error!("{panic_info}\n{backtrace}");

            // we also need to print to stderr directly for when using `$logTrace` because
            // the message won't be sent to the client.
            // But don't use `eprintln` because `eprintln` itself may panic if the pipe is broken.
            let mut stderr = std::io::stderr().lock();
            writeln!(stderr, "{panic_info}\n{backtrace}").ok();

            if let Some(client) = hook_client.upgrade() {
                client
                    .show_message(
                "The Fortitude language server exited with a panic. See the logs for more details."
                    .to_string(),
                lsp_types::MessageType::ERROR,
            )
                    .ok();
            }
        }));

        Self {
            hook: Some(hook),
            _client: client,
        }
    }
}
impl Drop for ServerPanicHookHandler {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Calling `std::panic::set_hook` while panicking results in a panic.
            return;
        }

        if let Some(hook) = self.hook.take() {
            std::panic::set_hook(hook);
        }
    }
}
