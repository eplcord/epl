use tracing::debug;
use axum::extract::ws::{Message, WebSocket, CloseFrame};

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

use futures::stream::SplitSink;
use futures::SinkExt;
use crate::AppState;
use crate::database::auth::{get_user_from_session};
use crate::database::entities::user::Model;
use crate::state::{GATEWAY_STATE, GatewayState};

pub async fn handle_identify(data: Identify, write: &mut SplitSink<WebSocket, Message>, state: &AppState) {
    debug!("Hello from handle_identify!");

    let user = match get_user_from_session(&state.conn, data.token).await {
        Ok(user) => user,
        Err(_) => {
            write.send(Message::Close(Some(CloseFrame { code: ErrorCode::AuthenticationFailed.into(), reason: ErrorCode::AuthenticationFailed.into() })))
                .await
                .expect("Failed to close websocket!");

            return;
        }
    };

    debug!("{}#{} ({}) has authed in handle_identify", &user.username, &user.discriminator, &user.id);

    // Initialise state
    let state = GATEWAY_STATE.get();

    state.set(GatewayState {
        user_id: user.id,
        bot: false,
        compress: false,
        large_threshold: 0,
        current_shard: 0,
        shard_count: 0,
        intents: 0,
        sender: write,
    });
}