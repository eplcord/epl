use std::sync::Arc;
use tracing::debug;
use axum::extract::ws::{Message, CloseFrame};
use tokio::sync::Mutex;

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

use crate::AppState;
use crate::database::auth::{get_user_from_session};
use crate::gateway::dispatch;
use crate::state::{GATEWAY_STATE, GatewayState, SOCKET};

pub async fn handle_identify(data: Identify, state: &AppState) {
    let mut socket = SOCKET.get().lock().await;

    debug!("Hello from handle_identify!");

    let user = match get_user_from_session(&state.conn, data.token).await {
        Ok(user) => user,
        Err(_) => {
            socket.inner.send(Message::Close(Some(CloseFrame { code: ErrorCode::AuthenticationFailed.into(), reason: ErrorCode::AuthenticationFailed.into() })))
                .await
                .expect("Failed to close websocket!");

            return;
        }
    };

    debug!("{}#{} ({}) has authed in handle_identify", &user.username, &user.discriminator, &user.id);

    // Initialise state
    let state = Arc::new(Mutex::new(GatewayState {
        user_id: user.id,
        bot: false,
        compress: false,
        large_threshold: 0,
        current_shard: 0,
        shard_count: 0,
        intents: 0
    }));

    GATEWAY_STATE.set(move || state.clone());

    drop(socket);
    dispatch::ready::dispatch_ready().await
}