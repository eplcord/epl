use tracing::debug;
use axum::extract::ws::{Message, WebSocket, CloseFrame};

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

use futures::stream::SplitSink;
use futures::SinkExt;

pub async fn handle_identify(data: Identify, write: &mut SplitSink<WebSocket, Message>) {
    debug!("Hello from handle_identify!");

    debug!("User token is: {}", data.token);

    write.send(Message::Close(Some(CloseFrame { code: ErrorCode::AuthenticationFailed.into(), reason: ErrorCode::AuthenticationFailed.into() })))
        .await
        .expect("Failed to close websocket!");
}