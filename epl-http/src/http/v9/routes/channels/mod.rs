pub mod pins;
pub mod attachments;
pub mod reactions;

use std::io;
use aws_sdk_s3::primitives::ByteStream;
use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::Utc;
use ril::{Image, Rgba};
use ril::ImageFormat::WebP;
use epl_common::database::entities::prelude::{Channel, ChannelMember, Embed, File, Mention, Message, MessageAttachment, Reaction, User};
use serde_derive::{Deserialize, Serialize};

use epl_common::nats::send_nats_message;
use epl_common::database::entities::{channel_member, embed, file, mention, message, message_attachment, pin, reaction, user};
use epl_common::messages::MessageTypes;
use epl_common::nats::Messages::{ChannelCreate, ChannelDelete, ChannelRecipientAdd, ChannelRecipientRemove, MessageCreate, MessageDelete, MessageUpdate, ProcessEmbed, TypingStarted};
use epl_common::rustflake::Snowflake;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use url::Url;
use epl_common::channels::ChannelTypes;
use epl_common::permissions::{internal_permission_calculator, InternalChannelPermissions};
use epl_common::relationship::get_relationship;
use epl_common::{RelationshipType, URL_REGEX, USER_MENTION_REGEX};
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::nats::Messages;
use epl_common::options::{EplOptions, Options};
use epl_common::schema::v9;
use epl_common::schema::v9::message::{generate_message_struct, generate_reactions, generate_refed_message};
use crate::http::v9::routes::users::channels::{ResChannel, ResChannelMember};


#[derive(Serialize)]
pub struct GetMessageRes(Vec<v9::message::Message>);

#[derive(Deserialize)]
pub struct GetMessageQuery {
    limit: Option<i32>,
    before: Option<i64>,
    after: Option<i64>,
    around: Option<i64>
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

            let messages: Vec<message::Model> = {
                if get_message_query.before.is_some() {
                    Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .filter(message::Column::Id.lt(get_message_query.before.unwrap()))
                        .order_by_desc(message::Column::Id)
                        .limit(limit as u64)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!")
                } else if get_message_query.after.is_some() {
                    // HACK: figure out a better way to do this possibly?
                    let mut messages = Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .filter(message::Column::Id.gt(get_message_query.after.unwrap()))
                        .order_by_asc(message::Column::Id)
                        .limit(limit as u64)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    messages.reverse();

                    messages
                } else if get_message_query.around.is_some() {
                    // HACK: this too
                    let mut before_messages = Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .filter(message::Column::Id.lt(get_message_query.around.unwrap()))
                        .limit(limit as u64 / 2 + 1)
                        .order_by_desc(message::Column::Id)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    let mut after_and_current_messages = Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .filter(message::Column::Id.gte(get_message_query.around.unwrap()))
                        .limit(limit as u64 / 2)
                        .order_by_asc(message::Column::Id)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    after_and_current_messages.reverse();

                    after_and_current_messages.append(&mut before_messages);

                    after_and_current_messages
                } else {
                    Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .limit(limit as u64)
                        .order_by_desc(message::Column::Id)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!")
                }
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

                let mentions: Vec<(mention::Model, Vec<user::Model>)> = Mention::find()
                    .filter(mention::Column::Message.eq(i.id))
                    .find_with_related(User)
                    .all(&state.conn)
                    .await
                    .expect("Failed to access database!");

                let mut mentioned_users = vec![];

                for x in mentions {
                    for e in x.1 {
                        mentioned_users.push(e);
                    }
                }

                let pinned = if let Some(_) = pin::Entity::find_by_id((i.channel_id, i.id))
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!") {
                    true
                } else {
                    false
                };

                let embeds = i.find_related(Embed).all(&state.conn).await.expect("Failed to access database!");

                let attachments = i.find_related(File).all(&state.conn).await.expect("Failed to access database!");

                let reactions = generate_reactions(&state.conn, &i, &session_context.user.id).await;

                output.push(generate_message_struct(i.clone(), author, refed_message, mentioned_users, pinned, embeds, attachments, reactions));
            }

            (StatusCode::OK, Json(GetMessageRes(output))).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SendMessageReq {
    content: String,
    flags: Option<i32>,
    nonce: String,
    tts: Option<bool>,
    message_reference: Option<v9::message::MessageReference>,
    allowed_mentions: Option<AllowedMentions>,
    mobile_network_type: Option<String>,
    attachments: Option<Vec<NewAttachment>>
}

#[derive(Deserialize)]
pub struct NewAttachment {
    filename: String,
    id: String,
    uploaded_filename: String,
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

            let mut mention_results = vec![];

            for i in USER_MENTION_REGEX.captures_iter(&message.content) {
                let user_id = i.get(1).unwrap().as_str().parse::<i64>();

                if user_id.is_err() {
                    continue;
                }

                let user_id = user_id.unwrap();

                let user = User::find()
                    .filter(user::Column::Id.eq(user_id))
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!");

                if user.is_none() {
                    continue;
                }

                mention_results.push(user.unwrap());
            }

            let new_message = message::Model {
                id: snowflake,
                channel_id: requested_channel.id,
                author: Some(session_context.user.id),
                content: message.content.clone(),
                timestamp: chrono::Utc::now().naive_utc(),
                edited_timestamp: None,
                tts: message.tts.unwrap_or(false),
                mention_everyone: calculated_permissions.contains(&InternalChannelPermissions::MentionEveryone) && message.content.contains("@everyone"),
                nonce: Some(message.nonce),
                r#type: {
                    if refed_message.is_some() {
                        MessageTypes::Reply as i32
                    } else {
                        MessageTypes::Default as i32
                    }
                },
                flags: Some(message.flags.unwrap_or(0)),
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

            for i in mention_results.as_slice() {
                Mention::insert(
                    mention::Model {
                        message: snowflake,
                        user: i.id,
                    }.into_active_model()
                )
                    .exec(&state.conn)
                    .await
                    .expect("Failed to access database!");
            }

            if message.attachments.is_some() & calculated_permissions.contains(&InternalChannelPermissions::AttachFiles) {
                for i in message.attachments.unwrap() {
                    MessageAttachment::insert(
                        message_attachment::Model {
                            message: snowflake,
                            file: i.uploaded_filename.parse().expect("Bad file id in attachments!"),
                        }.into_active_model()
                    )
                        .exec(&state.conn)
                        .await
                        .expect("Failed to access database!");
                }
            }

            send_nats_message(
                &state.nats_client,
                requested_channel.id.to_string(),
                MessageCreate { id: snowflake },
            )
                .await;

            if EplOptions::get().mediaproxy_url.is_some() {
                for i in URL_REGEX.captures_iter(&new_message.content) {
                    let url = Url::parse(i.get(0).unwrap().as_str());

                    if url.is_err() {
                        continue;
                    }

                    send_nats_message(
                        &state.nats_client,
                        "worker_queue".to_string(),
                        ProcessEmbed { message_id: new_message.id }
                    ).await;
                }
            }

            let attachments = new_message.find_related(File).all(&state.conn).await.expect("Failed to access database!");

            let reactions = generate_reactions(&state.conn, &new_message, &session_context.user.id).await;

            (
                StatusCode::OK,
                Json(generate_message_struct(
                    new_message,
                    Some(session_context.user),
                    refed_message,
                    mention_results,
                    false,
                    vec![],
                    attachments,
                    reactions
                )),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct EditMessageReq {
    content: Option<String>,
    attachments: Option<Vec<v9::message::Attachment>>,
    flags: Option<i32>
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

            // Remove all embeds
            if message.flags.is_some_and(|x| x == 4) {
                let embeds: Vec<embed::Model> = requested_message.find_related(Embed).all(&state.conn).await.expect("Failed to access database!");

                for i in embeds {
                    i.into_active_model().delete(&state.conn).await.expect("Failed to access database!");
                }
            }

            if message.attachments.is_some() {
                let mut ne_conditions = Condition::all();

                for i in message.attachments.unwrap() {
                    ne_conditions = ne_conditions.add(message_attachment::Column::File.ne(i.id.parse::<i64>().unwrap()));
                }

                let attachments: Vec<(message_attachment::Model, Vec<file::Model>)> = MessageAttachment::find()
                    .filter(message_attachment::Column::Message.eq(requested_message.id))
                    .filter(ne_conditions)
                    .find_with_related(File)
                    .all(&state.conn)
                    .await
                    .expect("Failed to access database!");

                for i in attachments {
                    for x in i.1 {
                        let mut x = x.into_active_model();

                        x.requested_deletion = Set(true);

                        x.update(&state.conn).await.expect("Failed to access database!");
                    }

                    i.0.into_active_model().delete(&state.conn).await.expect("Failed to access database!");
                }
            }

            let mut requested_message = requested_message.into_active_model();

            if message.content.is_some() {
                requested_message.content = Set(message.content.unwrap());
            }

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

            let mentions: Vec<mention::Model> = Mention::find()
                .filter(mention::Column::Message.eq(requested_message.id))
                .all(&state.conn)
                .await
                .expect("Failed to access database!");

            let mut mentioned_users = vec![];

            for i in mentions {
                let user = User::find_by_id(i.user)
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!");

                if user.is_none() {
                    continue;
                }

                mentioned_users.push(user.unwrap());
            }

            let pinned = if let Some(_) = pin::Entity::find_by_id((requested_message.channel_id, requested_message.id))
                .one(&state.conn)
                .await
                .expect("Failed to access database!") {
                true
            } else {
                false
            };

            let embeds = requested_message.find_related(Embed).all(&state.conn).await.expect("Failed to access database!");

            let attachments = requested_message.find_related(File).all(&state.conn).await.expect("Failed to access database!");

            let reactions = generate_reactions(&state.conn, &requested_message, &session_context.user.id).await;

            (
                StatusCode::OK,
                Json(generate_message_struct(
                    requested_message,
                    Some(session_context.user),
                    refed_message,
                    mentioned_users,
                    pinned,
                    embeds,
                    attachments,
                    reactions
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

            // Check to see if there's a pin, deleting it if there is
            if let Some(pin) = pin::Entity::find_by_id((cache.1, cache.0))
                .one(&state.conn)
                .await
                .expect("Failed to access database!") {
                pin
                    .delete(&state.conn)
                    .await
                    .expect("Failed to access database!");
            }

            // Delete any embeds attached to the message
            let embeds: Vec<embed::Model> = requested_message.find_related(Embed).all(&state.conn).await.expect("Failed to access database!");

            for i in embeds {
                i.delete(&state.conn).await.expect("Failed to access database!");
            }

            // Mark attachments as requesting deletion
            let attachments: Vec<(message_attachment::Model, Vec<file::Model>)> = MessageAttachment::find()
                .filter(message_attachment::Column::Message.eq(cache.0))
                .find_with_related(File)
                .all(&state.conn)
                .await
                .expect("Failed to access database!");

            for i in attachments {
                for x in i.1 {
                    let mut x = x.into_active_model();

                    x.requested_deletion = Set(true);

                    x.update(&state.conn).await.expect("Failed to access database!");
                }

                i.0.delete(&state.conn).await.expect("Failed to access database!");
            }

            let reactions: Vec<reaction::Model> = requested_message.find_related(Reaction).all(&state.conn).await.expect("Failed to access database!");

            for i in reactions {
                i.delete(&state.conn).await.expect("Failed to access database!");
            }

            requested_message
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
            ).await;

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

pub async fn add_user_to_channel(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((channel_id, user_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_channel) => {
            // Calculate permissions
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // Check if the user has permission to add users to the channel
            if !calculated_permissions.contains(&InternalChannelPermissions::AddMembers) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let requested_user = User::find_by_id(user_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!");

            match requested_user {
                None => StatusCode::BAD_REQUEST.into_response(),
                Some(requested_user) => {
                    // Check if the user is already a member of the channel
                    let channel_member = ChannelMember::find_by_id((requested_channel.id, requested_user.id))
                        .one(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    if channel_member.is_some() {
                        // User is already a member of the channel
                        StatusCode::BAD_REQUEST.into_response()
                    } else {
                        // If this is a group DM, check if the max number of users has been reached
                        // and also check if the users are friends
                        if requested_channel.r#type == (ChannelTypes::GroupDM as i32) {
                            let channel_members = ChannelMember::find()
                                .filter(channel_member::Column::Channel.eq(requested_channel.id))
                                .count(&state.conn)
                                .await
                                .expect("Failed to access database!");

                            if channel_members >= 10 {
                                // Max number of users has been reached
                                return StatusCode::BAD_REQUEST.into_response();
                            }

                            let relationship = get_relationship(session_context.user.id, requested_user.id, &state.conn).await;

                            if relationship.is_none() || relationship.unwrap().relationship_type != RelationshipType::Friend as i32 {
                                // Users are not friends
                                return StatusCode::BAD_REQUEST.into_response();
                            }
                        }

                        // Add the user to the channel
                        ChannelMember::insert(
                            channel_member::ActiveModel {
                                channel: Set(requested_channel.id),
                                user: Set(requested_user.id),
                            }
                        )
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        send_nats_message(
                            &state.nats_client,
                            requested_user.id.to_string(),
                            ChannelCreate { id: requested_channel.id }
                        ).await;

                        send_nats_message(
                            &state.nats_client,
                            requested_channel.id.to_string(),
                            ChannelRecipientAdd {
                                channel_id: requested_channel.id,
                                user_id: requested_user.id,
                            }
                        ).await;

                        // Create the arrival message
                        let snowflake = Snowflake::default().generate();

                        let new_message = message::Model {
                            id: snowflake,
                            channel_id: requested_channel.id,
                            author: Some(session_context.user.id),
                            content: String::new(),
                            timestamp: chrono::Utc::now().naive_utc(),
                            edited_timestamp: None,
                            tts: false,
                            mention_everyone: false,
                            nonce: None,
                            r#type: MessageTypes::RecipientAdd as i32,
                            flags: None,
                            reference_message_id: None,
                            reference_channel_id: None,
                            pinned: false,
                            webhook_id: None,
                            application_id: None,
                        };

                        Message::insert(new_message.clone().into_active_model())
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        Mention::insert(
                            mention::Model {
                                message: snowflake,
                                user: requested_user.id,
                            }.into_active_model()
                        )
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        send_nats_message(
                            &state.nats_client,
                            requested_channel.id.to_string(),
                            MessageCreate { id: snowflake },
                        )
                            .await;

                        StatusCode::NO_CONTENT.into_response()
                    }
                }
            }
        }
    }
}

pub async fn remove_user_from_channel(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path((channel_id, user_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(requested_channel) => {
            // Calculate permissions
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            // Check if the user has permission to remove users from the channel
            if !calculated_permissions.contains(&InternalChannelPermissions::KickMembers) {
                return StatusCode::BAD_REQUEST.into_response();
            }

            let requested_user = User::find_by_id(user_id)
                .one(&state.conn)
                .await
                .expect("Failed to access database!");

            match requested_user {
                None => StatusCode::BAD_REQUEST.into_response(),
                Some(requested_user) => {
                    // Check if the user is a member of the channel
                    let channel_member = ChannelMember::find_by_id((requested_channel.id, requested_user.id))
                        .one(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    if channel_member.is_none() {
                        // User is not a member of the channel
                        StatusCode::BAD_REQUEST.into_response()
                    } else {
                        // Remove the user from the channel
                        ChannelMember::delete(channel_member.unwrap().into_active_model())
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        send_nats_message(
                            &state.nats_client,
                            requested_user.id.to_string(),
                            ChannelDelete { id: requested_channel.id }
                        ).await;

                        send_nats_message(
                            &state.nats_client,
                            requested_channel.id.to_string(),
                            ChannelRecipientRemove {
                                channel_id: requested_channel.id,
                                user_id: requested_user.id,
                            }
                        ).await;

                        // Create the removal message
                        let snowflake = Snowflake::default().generate();

                        let new_message = message::Model {
                            id: snowflake,
                            channel_id: requested_channel.id,
                            author: Some(session_context.user.id),
                            content: String::new(),
                            timestamp: chrono::Utc::now().naive_utc(),
                            edited_timestamp: None,
                            tts: false,
                            mention_everyone: false,
                            nonce: None,
                            r#type: MessageTypes::RecipientRemove as i32,
                            flags: None,
                            reference_message_id: None,
                            reference_channel_id: None,
                            pinned: false,
                            webhook_id: None,
                            application_id: None,
                        };

                        Message::insert(new_message.clone().into_active_model())
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        Mention::insert(
                            mention::Model {
                                message: snowflake,
                                user: requested_user.id,
                            }.into_active_model()
                        )
                            .exec(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        send_nats_message(
                            &state.nats_client,
                            requested_channel.id.to_string(),
                            MessageCreate { id: snowflake },
                        ).await;

                        StatusCode::NO_CONTENT.into_response()
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct ModifyChannelReq {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub owner: Option<String>
}

pub async fn modify_channel(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
    Json(data): Json<ModifyChannelReq>
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

            let options = EplOptions::get();

            let mut active_channel = requested_channel.clone().into_active_model();
            let mut queued_messages: Vec<message::ActiveModel> = vec![];

            if let Some(name) = data.name {
                if !calculated_permissions.contains(&InternalChannelPermissions::EditName) {
                    return StatusCode::BAD_REQUEST.into_response();
                }

                let snowflake = Snowflake::default().generate();

                queued_messages.push(message::ActiveModel {
                    id: Set(snowflake),
                    channel_id: Set(requested_channel.id),
                    author: Set(Some(session_context.user.id)),
                    content: Set(name.clone()),
                    timestamp: Set(chrono::Utc::now().naive_utc()),
                    r#type: Set(MessageTypes::ChannelNameChange as i32),
                    tts: Set(false),
                    mention_everyone: Set(false),
                    pinned: Set(false),
                    ..Default::default()
                });

                active_channel.name = Set(Some(name));
            }

            if let Some(owner) = data.owner {
                if requested_channel.owner_id.is_some_and(|x| x.eq(&session_context.user.id)) {
                    active_channel.owner_id = Set(Some(owner.parse().unwrap()));
                } else {
                    return StatusCode::BAD_REQUEST.into_response();
                }
            }

            if let Some(icon) = data.icon {
                if !calculated_permissions.contains(&InternalChannelPermissions::EditIcon) {
                    return StatusCode::BAD_REQUEST.into_response();
                }

                let snowflake = Snowflake::default().generate();

                queued_messages.push(message::ActiveModel {
                    id: Set(snowflake),
                    channel_id: Set(requested_channel.id),
                    author: Set(Some(session_context.user.id)),
                    content: Set("".to_string()),
                    timestamp: Set(chrono::Utc::now().naive_utc()),
                    r#type: Set(MessageTypes::ChannelIconChange as i32),
                    tts: Set(false),
                    mention_everyone: Set(false),
                    pinned: Set(false),
                    ..Default::default()
                });

                let image_bytes = icon.split("base64,").collect::<Vec<&str>>()[1].as_bytes();
                let image = BASE64_STANDARD.decode(image_bytes).expect("Invalid base64! Bailing!");

                let hash = sha256::digest(&image);

                let mut image_buffer: Vec<u8> = Vec::new();
                let image: Image<Rgba> = Image::from_reader_inferred(&mut io::Cursor::new(image)).expect("Invalid image!");
                image.encode(WebP, &mut image_buffer).expect("Failed to encode image!");

                let s3_res = state.aws.put_object()
                    .bucket(options.s3_bucket)
                    .key(format!("channel-icons/{}/{hash}.webp", active_channel.clone().id.unwrap()))
                    .body(ByteStream::from(image_buffer))
                    .send()
                    .await;

                match s3_res {
                    Ok(_) => {
                        active_channel.icon = Set(Some(hash.to_string()))
                    }
                    Err(_) => {
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }

            let mut users: Vec<ResChannelMember> = vec![];

            for (_chnlmem, usr) in ChannelMember::find()
                .filter(channel_member::Column::Channel.eq(requested_channel.id))
                .find_also_related(User)
                .all(&state.conn)
                .await
                .expect("Failed to access database!") {

                match usr {
                    None => {
                        // Channel member references non existent user?
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                    Some(usr) => {
                        users.push(ResChannelMember {
                            accent_color: usr.accent_color.map(|e| {
                                e.parse().expect("Failed to parse user's accent_color")
                            }),
                            avatar: usr.avatar,
                            avatar_decoration: usr.avatar_decoration,
                            banner: usr.banner,
                            banner_color: usr.banner_colour,
                            discriminator: Some(usr.discriminator),
                            flags: generate_public_flags(get_user_flags(usr.flags)),
                            global_name: usr.display_name,
                            id: usr.id.to_string(),
                            public_flags: generate_public_flags(get_user_flags(usr.flags)),
                            username: usr.username,
                        })
                    }
                }
            }

            match active_channel.update(&state.conn).await {
                Ok(channel) => {
                    for i in queued_messages {
                        let message = i.insert(&state.conn).await.expect("Failed to insert channel update message!");

                        send_nats_message(&state.nats_client, channel.id.to_string(), Messages::MessageCreate { id: message.id }).await;
                    }

                    send_nats_message(&state.nats_client, channel.id.to_string(), Messages::ChannelUpdate { channel_id: channel.id }).await;

                    Json(ResChannel {
                        flags: channel.flags.unwrap_or(0),
                        id: channel.id.to_string(),
                        icon: channel.icon,
                        last_message_id: channel.last_message_id.map(|x| x.to_string()),
                        name: channel.name,
                        owner_id: channel.owner_id.map(|x| x.to_string()),
                        recipients: Some(users),
                        _type: channel.r#type,
                    }).into_response()
                }
                Err(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }
}