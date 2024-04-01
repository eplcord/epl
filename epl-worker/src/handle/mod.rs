mod process_embed;

use tracing::log::error;
use epl_common::nats::Messages;
use crate::AppState;
use crate::handle::process_embed::process_embed;

pub async fn handle_nats_message(state: &AppState, message: Messages) {
    match message {
        Messages::ProcessEmbed { message_id } => {
            process_embed(state, message_id).await;
        }
        _ => {
            error!("Unsupported message received!");
        }
    }
}