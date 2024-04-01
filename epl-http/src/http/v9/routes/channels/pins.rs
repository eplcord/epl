use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder};
use tracing::error;
use epl_common::database::entities::{mention, message, pin, user};
use epl_common::database::entities::prelude::{Channel, Embed, Mention, Message, Pin, User};
use epl_common::messages::MessageTypes;
use epl_common::nats::Messages::{ChannelPinsAck, ChannelPinsUpdate, MessageCreate, MessageUpdate};
use epl_common::permissions::{internal_permission_calculator, InternalChannelPermissions};
use epl_common::rustflake::Snowflake;
use crate::AppState;
use crate::authorization_extractor::SessionContext;
use crate::http::v9::{generate_message_struct, generate_refed_message, SharedMessage};
use epl_common::nats::send_nats_message;

pub async fn new_pin(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((channel_id, message_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => {
            StatusCode::BAD_REQUEST
        }
        Some(requested_channel) => {
            let requested_message = Message::find_by_id(message_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!");

            match requested_message {
                None => {
                    StatusCode::BAD_REQUEST
                }
                Some(requested_message) => {
                    let calculated_permissions = internal_permission_calculator(
                        &requested_channel,
                        &session_context.user,
                        Some(&requested_message),
                        &state.conn
                    ).await;

                    if !calculated_permissions.contains(&InternalChannelPermissions::PinMessage) {
                        return StatusCode::BAD_REQUEST;
                    }

                    let new_pin = pin::ActiveModel{
                        channel: Set(requested_channel.id),
                        message: Set(requested_message.id),
                        timestamp: Set(chrono::Utc::now().naive_utc())
                    };

                    match new_pin.insert(&state.conn).await {
                        Ok(new_pin) => {
                            // Create the pin created message
                            let snowflake = Snowflake::default().generate();

                            let new_pin_created_message = message::Model {
                                id: snowflake,
                                channel_id: requested_channel.id,
                                author: Some(session_context.user.id),
                                content: String::new(),
                                timestamp: chrono::Utc::now().naive_utc(),
                                edited_timestamp: None,
                                tts: false,
                                mention_everyone: false,
                                nonce: None,
                                r#type: MessageTypes::ChannelPinnedMessage as i32,
                                flags: None,
                                reference_message_id: Some(requested_message.id),
                                reference_channel_id: Some(requested_channel.id),
                                pinned: false,
                                webhook_id: None,
                                application_id: None,
                            };

                            Message::insert(new_pin_created_message.clone().into_active_model())
                                .exec(&state.conn)
                                .await
                                .expect("Failed to access database!");

                            send_nats_message(
                                &state.nats_client,
                                requested_channel.id.to_string(),
                                MessageCreate { id: snowflake },
                            ).await;

                            send_nats_message(
                                &state.nats_client,
                                requested_channel.id.to_string(),
                                ChannelPinsUpdate {
                                    channel_id: new_pin.channel,
                                }
                            ).await;

                            send_nats_message(
                                &state.nats_client,
                                session_context.user.id.to_string(),
                                ChannelPinsAck {
                                    channel_id:  requested_channel.id,
                                }
                            ).await;

                            send_nats_message(
                                &state.nats_client,
                                requested_channel.id.to_string(),
                                MessageUpdate {
                                    id: requested_message.id,
                                }
                            ).await;

                            StatusCode::NO_CONTENT
                        }
                        Err(_) => {
                            StatusCode::INTERNAL_SERVER_ERROR
                        }
                    }
                }
            }
        }
    }
}

pub async fn delete_pin(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((channel_id, message_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => {
            StatusCode::BAD_REQUEST
        }
        Some(requested_channel) => {
            let requested_message = Message::find_by_id(message_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!");

            match requested_message {
                None => {
                    StatusCode::BAD_REQUEST
                }
                Some(requested_message) => {
                    let calculated_permissions = internal_permission_calculator(
                        &requested_channel,
                        &session_context.user,
                        Some(&requested_message),
                        &state.conn
                    ).await;

                    if !calculated_permissions.contains(&InternalChannelPermissions::PinMessage) {
                        return StatusCode::BAD_REQUEST;
                    }

                    let pin = Pin::find_by_id((requested_channel.id, requested_message.id))
                        .one(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    match pin {
                        None => {
                            StatusCode::BAD_REQUEST
                        }
                        Some(pin) => {
                            pin.delete(&state.conn).await.expect("Failed to access database!");

                            send_nats_message(
                                &state.nats_client,
                                requested_channel.id.to_string(),
                                ChannelPinsUpdate {
                                    channel_id: requested_channel.id,
                                }
                            ).await;

                            send_nats_message(
                                &state.nats_client,
                                session_context.user.id.to_string(),
                                ChannelPinsAck {
                                    channel_id:  requested_channel.id,
                                }
                            ).await;

                            send_nats_message(
                                &state.nats_client,
                                requested_channel.id.to_string(),
                                MessageUpdate {
                                    id: requested_message.id,
                                }
                            ).await;

                            StatusCode::NO_CONTENT
                        }
                    }
                }
            }
        }
    }
}

pub async fn get_pins(
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
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // This isn't *exactly* how Discord handles this but is a good approximation (and imo how it *should* be handled)
            if !calculated_permissions.contains(&InternalChannelPermissions::ViewHistory) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let mut pins: Vec<SharedMessage> = vec![];
            let pins_model: Vec<(pin::Model, Option<message::Model>)> = Pin::find()
                .filter(pin::Column::Channel.eq(requested_channel.id))
                .order_by_desc(pin::Column::Timestamp)
                .find_also_related(Message)
                .all(&state.conn)
                .await
                .expect("Failed to access database!");

            for i in pins_model {
                match i.1 {
                    None => {
                        error!("Pin references non existent message!");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                    Some(message) => {
                        let author = User::find_by_id(message.author.unwrap_or(0))
                            .one(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

                        if message.reference_message_id.is_some() {
                            refed_message =
                                generate_refed_message(&state.conn, message.reference_message_id.unwrap()).await;
                        }

                        let mentions: Vec<(mention::Model, Vec<user::Model>)> = Mention::find()
                            .filter(mention::Column::Message.eq(message.id))
                            .find_with_related(User)
                            .all(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        let mut mentioned_users = vec![];

                        for i in mentions {
                            for x in i.1 {
                                mentioned_users.push(x);
                            }
                        }

                        let embeds = message.find_related(Embed).all(&state.conn).await.expect("Failed to access database!");

                        pins.push(generate_message_struct(message, author, refed_message, mentioned_users, true, embeds));
                    }
                }
            }

            Json(pins).into_response()
        }
    }
}