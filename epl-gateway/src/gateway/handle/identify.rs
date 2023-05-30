use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbErr, IntoActiveModel};
use tracing::debug;

use crate::gateway::schema::identify::Identify;

use crate::gateway::dispatch;
use crate::state::{GatewayState, EncodingType, CompressionType, ThreadData};
use crate::AppState;
use epl_common::database::auth::{get_session_by_token, get_user_from_session_by_token, GetSessionError};
use epl_common::get_location_from_ip;
use crate::gateway::dispatch::send_close;
use crate::gateway::schema::error_codes::ErrorCode::AuthenticationFailed;

pub async fn handle_identify(thread_data: &mut ThreadData, data: Identify, state: &AppState) {
    debug!("Hello from handle_identify!");

    let user = match get_user_from_session_by_token(&state.conn, &data.token).await {
        Ok(user) => user,
        Err(_) => {
            send_close(thread_data, AuthenticationFailed).await;
            return;
        }
    };

    debug!(
        "{}#{} ({}) has authed in handle_identify",
        &user.username, &user.discriminator, &user.id
    );

    // Initialise state

    let compression_encoding: (Option<CompressionType>, EncodingType) = {
        let gateway_state = thread_data.gateway_state.as_mut().unwrap();

        let compression_type = gateway_state.compression.clone();
        let encoding_type = gateway_state.encoding.clone();

        (compression_type, encoding_type)
    };

    let mut session = match get_session_by_token(&state.conn, &data.token).await {
        Ok(session) => session,
        Err(_) => {
            send_close(thread_data, AuthenticationFailed).await;
            return;
        }
    }.into_active_model();

    session.last_used = Set(Utc::now().naive_utc());

    let props = data.properties.unwrap();

    session.os = Set(props.os);
    session.platform = Set(props.browser);

    session.location = Set(Some(get_location_from_ip(thread_data.session_ip.unwrap())));

    session.update(&state.conn).await.expect("Failed to update session with props");

    // TODO: calculate these
    let gateway_state = Some(GatewayState {
        user_id: Some(user.id),
        bot: Some(user.bot),
        large_threshold: Some(50),
        current_shard: Some(0),
        shard_count: Some(0),
        intents: Some(0),
        compression: compression_encoding.0,
        encoding: compression_encoding.1,
    });
    thread_data.gateway_state = gateway_state;

    dispatch::ready::dispatch_ready(thread_data, user, &data.token, state).await;
}
