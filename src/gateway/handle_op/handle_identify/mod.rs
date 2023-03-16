use tracing::debug;
use axum::extract::ws::{Message, WebSocket, CloseFrame};

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

use futures::stream::SplitSink;
use futures::SinkExt;
use crate::AppState;
use crate::database::auth::{get_user_from_session, GetSessionError};
use crate::database::entities::user::Model;

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

    write.send(Message::Close(Some(CloseFrame { code: ErrorCode::AuthenticationFailed.into(), reason: ErrorCode::AuthenticationFailed.into() })))
        .await
        .expect("Failed to close websocket!");
}