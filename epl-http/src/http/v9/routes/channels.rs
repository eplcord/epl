use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::Utc;
use epl_common::database::entities::prelude::{Channel, Message, User};
use serde_derive::{Deserialize, Serialize};

use crate::http::v9::{
    generate_message_struct, generate_refed_message, SharedMessage, SharedMessageReference,
};
use crate::nats::send_nats_message;
use epl_common::database::entities::{message, user};
use epl_common::messages::MessageTypes;
use epl_common::nats::Messages::{MessageCreate, MessageDelete, MessageUpdate, TypingStarted};
use epl_common::rustflake::Snowflake;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use epl_common::permissions::{internal_permission_calculator, InternalChannelPermissions};

#[derive(Serialize)]
pub struct GetMessageRes(Vec<SharedMessage>);

#[derive(Deserialize)]
pub struct GetMessageQuery {
    limit: Option<i32>,
    before: Option<i64>,
}

pub async fn get_messages(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
    Query(get_message_query): Query<GetMessageQuery>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_channel) => {
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // Check if the user has permission to view message history (this is just a guess on what is returned)
            // TODO: Investigate what status code is actually returned for this
            if !calculated_permissions.contains(&InternalChannelPermissions::ViewHistory) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let mut output = vec![];

            let limit = get_message_query.limit.unwrap_or(50);

            let messages: Vec<message::Model> = match get_message_query.before {
                None => Message::find()
                    .filter(message::Column::ChannelId.eq(requested_channel.id))
                    .limit(limit as u64)
                    .order_by_desc(message::Column::Id)
                    .all(&state.conn)
                    .await
                    .expect("Failed to access database!"),
                Some(before) => Message::find()
                    .filter(message::Column::ChannelId.eq(requested_channel.id))
                    .limit(limit as u64)
                    .order_by_desc(message::Column::Id)
                    .cursor_by(message::Column::Id)
                    .before(before)
                    .all(&state.conn)
                    .await
                    .expect("Failed to access database!"),
            };

            for i in messages {
                let author = User::find_by_id(i.author.unwrap_or(0))
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!");

                let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

                if i.reference_message_id.is_some() {
                    refed_message =
                        generate_refed_message(&state.conn, i.reference_message_id.unwrap()).await;
                }

                output.push(generate_message_struct(i.clone(), author, refed_message));
            }

            (StatusCode::OK, Json(GetMessageRes(output))).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SendMessageReq {
    content: String,
    flags: i32,
    nonce: String,
    tts: bool,
    message_reference: Option<SharedMessageReference>,
    allowed_mentions: Option<AllowedMentions>,
}

#[derive(Deserialize)]
pub struct AllowedMentions {
    parse: Vec<String>,
    replied_user: bool,
}

pub async fn send_message(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
    Json(message): Json<SendMessageReq>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_channel) => {
            let snowflake = Snowflake::default().generate();

            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // Check if the user has permission to send messages
            if !calculated_permissions.contains(&InternalChannelPermissions::SendMessage) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

            if message.message_reference.is_some() {
                refed_message = generate_refed_message(
                    &state.conn,
                    message
                        .message_reference
                        .unwrap()
                        .message_id
                        .parse::<i64>()
                        .unwrap(),
                )
                .await;
            }

            let new_message = message::Model {
                id: snowflake,
                channel_id: requested_channel.id,
                author: Some(session_context.user.id),
                content: message.content.clone(),
                timestamp: chrono::Utc::now().naive_utc(),
                edited_timestamp: None,
                tts: message.tts,
                mention_everyone: message.content.contains("@everyone"),
                nonce: Some(message.nonce),
                r#type: {
                    if refed_message.is_some() {
                        MessageTypes::Reply as i32
                    } else {
                        MessageTypes::Default as i32
                    }
                },
                flags: Some(message.flags),
                reference_message_id: if let Some(message_ref) = refed_message.clone() {
                    Some(message_ref.0.id)
                } else {
                    None
                },
                reference_channel_id: if let Some(message_ref) = refed_message.clone() {
                    Some(message_ref.0.channel_id)
                } else {
                    None
                },
                pinned: false,
                webhook_id: None,
                application_id: None,
            };

            Message::insert(new_message.clone().into_active_model())
                .exec(&state.conn)
                .await
                .expect("Failed to access database!");

            send_nats_message(
                &state.nats_client,
                requested_channel.id.to_string(),
                MessageCreate { id: snowflake },
            )
            .await;

            (
                StatusCode::OK,
                Json(generate_message_struct(
                    new_message,
                    Some(session_context.user),
                    refed_message,
                )),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct EditMessageReq {
    content: String,
}

pub async fn edit_message(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id)): Path<(i64, i64)>,
    Json(message): Json<EditMessageReq>,
) -> impl IntoResponse {
    // Ensure message actually exists
    let requested_message = Message::find_by_id(message_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_message) => {
            // Calculate permissions
            let calculated_permissions = internal_permission_calculator(
                &Channel::find_by_id(requested_message.channel_id)
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!")
                    .expect("Message references non-existent channel!"),
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            // Check if the user has permission to edit the message
            if !calculated_permissions.contains(&InternalChannelPermissions::EditMessage) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let mut requested_message = requested_message.into_active_model();

            requested_message.content = Set(message.content);
            requested_message.edited_timestamp = Set(Some(chrono::Utc::now().naive_utc()));

            let requested_message = requested_message
                .update(&state.conn)
                .await
                .expect("Failed to access database!");

            send_nats_message(
                &state.nats_client,
                requested_message.channel_id.to_string(),
                MessageUpdate {
                    id: requested_message.id,
                },
            )
            .await;

            let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

            if requested_message.reference_message_id.is_some() {
                refed_message = generate_refed_message(
                    &state.conn,
                    requested_message.reference_message_id.unwrap(),
                )
                .await;
            }

            (
                StatusCode::OK,
                Json(generate_message_struct(
                    requested_message,
                    Some(session_context.user),
                    refed_message,
                )),
            )
                .into_response()
        }
    }
}

pub async fn delete_message(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Ensure message actually exists
    let requested_message = Message::find_by_id(message_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_message) => {
            let channel = Channel::find_by_id(requested_message.channel_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Message references non-existent channel!");

            // Calculate permissions
            let calculated_permissions = internal_permission_calculator(
                &channel,
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            // Check if the user has permission to delete the message
            if !calculated_permissions.contains(&InternalChannelPermissions::DeleteMessage) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let cache: (i64, i64, Option<i64>) =
                (requested_message.id, channel.id, channel.guild_id);

            requested_message
                .into_active_model()
                .delete(&state.conn)
                .await
                .expect("Failed to access database!");

            send_nats_message(
                &state.nats_client,
                cache.1.to_string(),
                MessageDelete {
                    id: cache.0,
                    channel_id: cache.1,
                    guild_id: cache.2,
                },
            )
            .await;

            StatusCode::NO_CONTENT.into_response()
        }
    }
}

pub async fn typing(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_channel) => {
            // Calculate permissions
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // Check if the user has permission to send messages (and actually trigger the typing event)
            if !calculated_permissions.contains(&InternalChannelPermissions::SendMessage) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            send_nats_message(
                &state.nats_client,
                channel_id.to_string(),
                TypingStarted {
                    channel_id,
                    user_id: session_context.user.id,
                    timestamp: Utc::now().naive_utc(),
                }
            ).await;

            StatusCode::NO_CONTENT.into_response()
        }
    }
}