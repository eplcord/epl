use axum::extract::ws::{CloseFrame, Message};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

use crate::gateway::dispatch;
use crate::state::{GatewayState, GATEWAY_STATE, SOCKET};
use crate::AppState;
use epl_common::database::auth::get_user_from_session;

pub async fn handle_identify(data: Identify, state: &AppState) {
    let mut socket = SOCKET.get().lock().await;

    debug!("Hello from handle_identify!");

    let user = match get_user_from_session(&state.conn, &data.token).await {
        Ok(user) => user,
        Err(_) => {
            socket
                .as_mut()
                .unwrap()
                .inner
                .send(Message::Close(Some(CloseFrame {
                    code: ErrorCode::AuthenticationFailed.into(),
                    reason: ErrorCode::AuthenticationFailed.into(),
                })))
                .await
                .expect("Failed to close websocket!");

            return;
        }
    };

    debug!(
        "{}#{} ({}) has authed in handle_identify",
        &user.username, &user.discriminator, &user.id
    );

    // Initialise state

    // TODO: calculate these

    let gateway_state = Arc::new(Mutex::new(Some(GatewayState {
        user_id: user.id,
        bot: user.bot,
        compress: data.compress.unwrap_or(false),
        large_threshold: 50,
        current_shard: 0,
        shard_count: 0,
        intents: 0,
    })));
    let gateway_state_c = gateway_state.clone();

    if !GATEWAY_STATE.set(move || gateway_state_c.clone()) {
        let inner = gateway_state.lock().await.take();
        GATEWAY_STATE.get().lock().await.replace(inner.unwrap());
    }

    drop(socket);
    dispatch::ready::dispatch_ready(user, &data.token, state).await;
}
