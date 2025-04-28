use anyhow::Context;
use lsp_types::notification::Notification;
use std::sync::OnceLock;

use crate::server::ClientSender;

static MESSENGER: OnceLock<ClientSender> = OnceLock::new();

pub(crate) fn init_messenger(client_sender: ClientSender) {
    MESSENGER
        .set(client_sender)
        .expect("messenger should only be initialized once");
}

pub(super) fn try_show_message(
    message: String,
    message_type: lsp_types::MessageType,
) -> crate::Result<()> {
    MESSENGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("messenger not initialized"))?
        .send(lsp_server::Message::Notification(
            lsp_server::Notification {
                method: lsp_types::notification::ShowMessage::METHOD.into(),
                params: serde_json::to_value(lsp_types::ShowMessageParams {
                    typ: message_type,
                    message,
                })?,
            },
        ))
        .context("Failed to send message")?;

    Ok(())
}
