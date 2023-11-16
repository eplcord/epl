use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use tracing::debug;

use crate::gateway::schema::identify::Identify;

use crate::gateway::dispatch;
use crate::gateway::dispatch::send_close;
use crate::gateway::schema::error_codes::ErrorCode::AuthenticationFailed;
use crate::state::{CompressionType, EncodingType, GatewayState, ThreadData};
use crate::AppState;
use epl_common::database::auth::{get_session_by_token, get_user_from_session_by_token};
use epl_common::get_location_from_ip;

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

    let current_gateway_state: (i64, Option<CompressionType>, EncodingType) = {
        let gateway_session_id = thread_data.gateway_state.gateway_session_id;
        let compression_type = thread_data.gateway_state.compression.clone();
        let encoding_type = thread_data.gateway_state.encoding.clone();

        (gateway_session_id, compression_type, encoding_type)
    };

    let mut session = match get_session_by_token(&state.conn, &data.token).await {
        Ok(session) => session,
        Err(_) => {
            send_close(thread_data, AuthenticationFailed).await;
            return;
        }
    }
    .into_active_model();

    session.last_used = Set(Utc::now().naive_utc());

    if let Some(props) = data.properties {
        session.os = Set(props.os);
        session.platform = Set(props.browser);
    }

    session.location = Set(Some(get_location_from_ip(thread_data.session_ip)));

    let session_id = session.clone().session_id.unwrap();

    session
        .update(&state.conn)
        .await
        .expect("Failed to update session with props");

    // TODO: calculate these
    let gateway_state = GatewayState {
        gateway_session_id: current_gateway_state.0,
        user_id: Some(user.id),
        session_id: Some(session_id),
        bot: Some(user.bot),
        large_threshold: Some(50),
        current_shard: Some(0),
        shard_count: Some(0),
        intents: Some(0),
        compression: current_gateway_state.1,
        encoding: current_gateway_state.2,
    };
    thread_data.gateway_state = gateway_state;

    thread_data.nats_subscriptions.insert(
        format!("{}", thread_data.gateway_state.user_id.unwrap()),
        thread_data
            .nats
            .subscribe(format!("{}", thread_data.gateway_state.user_id.unwrap()))
            .await
            .expect("Failed to subscribe!"),
    );

    dispatch::ready::dispatch_ready(thread_data, user, &data.token, state).await;
}
