use axum::extract::ws::{CloseFrame, Message};
use crate::gateway::schema::error_codes::ErrorCode;
use crate::state::SOCKET;



pub async fn dispatch_ready() {
    let mut socket = SOCKET.get().lock().await;

    socket.inner.send(Message::Close(Some(CloseFrame { code: ErrorCode::AuthenticationFailed.into(), reason: ErrorCode::AuthenticationFailed.into() })))
        .await
        .expect("Failed to close websocket!");
}