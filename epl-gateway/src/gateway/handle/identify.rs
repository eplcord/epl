use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::gateway::schema::identify::Identify;

use crate::gateway::dispatch;
use crate::state::{GatewayState, GATEWAY_STATE, EncodingType, CompressionType};
use crate::AppState;
use epl_common::database::auth::get_user_from_session;
use crate::gateway::dispatch::send_close;
use crate::gateway::schema::error_codes::ErrorCode::AuthenticationFailed;

pub async fn handle_identify(data: Identify, state: &AppState) {
    debug!("Hello from handle_identify!");

    let user = match get_user_from_session(&state.conn, &data.token).await {
        Ok(user) => user,
        Err(_) => {
            send_close(AuthenticationFailed).await;
            return;
        }
    };

    debug!(
        "{}#{} ({}) has authed in handle_identify",
        &user.username, &user.discriminator, &user.id
    );

    // Initialise state

    let compression_encoding: (Option<CompressionType>, EncodingType) = {
        let mut gateway_state_lock = GATEWAY_STATE.get().lock().await;
        let gateway_state = gateway_state_lock.as_mut().unwrap();

        let compression_type = gateway_state.compression.clone();
        let encoding_type = gateway_state.encoding.clone();

        drop(gateway_state_lock);

        (compression_type, encoding_type)
    };

    // TODO: calculate these
    let gateway_state = Arc::new(Mutex::new(Some(GatewayState {
        user_id: Some(user.id),
        bot: Some(user.bot),
        large_threshold: Some(50),
        current_shard: Some(0),
        shard_count: Some(0),
        intents: Some(0),
        compression: compression_encoding.0,
        encoding: compression_encoding.1,
    })));
    let gateway_state_c = gateway_state.clone();

    if !GATEWAY_STATE.set(move || gateway_state_c.clone()) {
        let inner = gateway_state.lock().await.take();
        GATEWAY_STATE.get().lock().await.replace(inner.unwrap());
    }

    dispatch::ready::dispatch_ready(user, &data.token, state).await;
}
