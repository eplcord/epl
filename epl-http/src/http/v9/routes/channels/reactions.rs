use axum::{Extension, Json};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QuerySelect};
use sea_orm::ActiveValue::Set;
use serde_derive::Deserialize;
use unic::emoji::char::is_emoji;
use epl_common::database::entities::prelude::{Channel, Message, Reaction, User};
use epl_common::database::entities::reaction;
use epl_common::nats::Messages::{MessageReactionAdd, MessageReactionRemove};
use epl_common::nats::send_nats_message;
use epl_common::permissions::{internal_permission_calculator, InternalChannelPermissions};
use epl_common::schema::v9;
use epl_common::schema::v9::user::generate_user_struct;
use crate::AppState;
use crate::authorization_extractor::SessionContext;

#[derive(Deserialize)]
pub struct GetReactionsQuery {
    limit: u64,
    #[serde(rename = "type")]
    burst: i32
}

pub async fn get_reactions(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id, emoji)): Path<(i64, i64, String)>,
    Query(params): Query<GetReactionsQuery>,
) -> impl IntoResponse {
    let burst = params.burst == 1;

    let requested_message = Message::find_by_id(message_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_message) => {
            let requested_channel = Channel::find_by_id(requested_message.channel_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Message references non-existent channel!");

            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            if !calculated_permissions.contains(&InternalChannelPermissions::ViewChannel) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let all_reactors_of_emoji = Reaction::find()
                .filter(reaction::Column::Message.eq(requested_message.id))
                .filter(reaction::Column::Emoji.eq(emoji))
                .filter(reaction::Column::Burst.eq(burst))
                .limit(params.limit)
                .all(&state.conn)
                .await
                .expect("Failed to access database!");

            let mut users_reacted: Vec<v9::user::User> = vec![];

            for i in all_reactors_of_emoji {
                let user = i.find_related(User)
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!")
                    .expect("Reaction references invalid user!");

                users_reacted.push(generate_user_struct(user));
            }

            Json(users_reacted).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AddReactionQuery {
    #[serde(rename = "location")]
    _location: String,
    #[serde(rename = "type")]
    burst: i32
}

pub async fn add_reaction(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id, emoji)): Path<(i64, i64, String)>,
    Query(params): Query<AddReactionQuery>,
) -> impl IntoResponse {
    let requested_message = Message::find_by_id(message_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_message) => {
            let requested_channel = Channel::find_by_id(requested_message.channel_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Message references non-existent channel!");

            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            if !calculated_permissions.contains(&InternalChannelPermissions::AddReactions) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let emoji = emoji.chars().next().unwrap();

            // TODO: implement guild emojis
            if !is_emoji(emoji) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let new_reaction = reaction::ActiveModel {
                user: Set(session_context.user.id),
                message: Set(requested_message.id),
                emoji: Set(emoji.to_string()),
                burst: Set(params.burst == 1),
            };

            new_reaction.insert(&state.conn).await.expect("Failed to access database!");

            send_nats_message(
                &state.nats_client,
                requested_channel.id.to_string(),
                MessageReactionAdd {
                    message_id: requested_message.id,
                    user_id: session_context.user.id,
                    emoji: emoji.to_string(),
                }
            ).await;

            StatusCode::NO_CONTENT.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RemoveReactionQuery {
    #[serde(rename = "location")]
    _location: String,
    #[serde(rename = "burst")]
    _burst: bool
}

pub async fn remove_reaction(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id, emoji, _burst)): Path<(i64, i64, String, i32)>,
    Query(_params): Query<RemoveReactionQuery>,
) -> impl IntoResponse {
    let requested_reaction = Reaction::find_by_id((session_context.user.id, message_id, emoji))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_reaction {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_reaction) => {
            let requested_message = Message::find_by_id(requested_reaction.message)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Reaction references non-existent message!");

            let requested_channel = Channel::find_by_id(requested_message.channel_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Message references non-existent channel!");

            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            if !calculated_permissions.contains(&InternalChannelPermissions::ViewChannel) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            send_nats_message(
                &state.nats_client,
                requested_channel.id.to_string(),
                MessageReactionRemove {
                    message_id: requested_message.id,
                    user_id: session_context.user.id,
                    emoji: requested_reaction.emoji.clone(),
                }
            ).await;

            requested_reaction.into_active_model().delete(&state.conn).await.expect("Failed to access database!");

            StatusCode::NO_CONTENT.into_response()
        }
    }
}

pub async fn delete_specific_user_reaction(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((_channel_id, message_id, emoji, _burst, user_id)): Path<(i64, i64, String, i32, i64)>,
    Query(_params): Query<RemoveReactionQuery>,
) -> impl IntoResponse {
    let requested_reaction = Reaction::find_by_id((user_id, message_id, emoji))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_reaction {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_reaction) => {
            let requested_message = Message::find_by_id(requested_reaction.message)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Reaction references non-existent message!");

            let requested_channel = Channel::find_by_id(requested_message.channel_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Message references non-existent channel!");

            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                Some(&requested_message),
                &state.conn
            ).await;

            if !calculated_permissions.contains(&InternalChannelPermissions::EditMessage) && session_context.user.id != requested_reaction.user {
                return StatusCode::BAD_REQUEST.into_response();
            }

            send_nats_message(
                &state.nats_client,
                requested_channel.id.to_string(),
                MessageReactionRemove {
                    message_id: requested_message.id,
                    user_id: requested_reaction.user,
                    emoji: requested_reaction.emoji.clone(),
                }
            ).await;

            requested_reaction.into_active_model().delete(&state.conn).await.expect("Failed to access database!");

            StatusCode::NO_CONTENT.into_response()
        }
    }
}