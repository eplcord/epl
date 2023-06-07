use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::prelude::{Channel, ChannelMember, Message, User};

use sea_orm::*;
use sea_orm::ActiveValue::Set;
use epl_common::database::entities::{channel_member, message, user};
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::messages::MessageTypes;
use epl_common::nats::Messages::MessageCreate;
use epl_common::rustflake::Snowflake;
use crate::http::v9::{SharedMessage, SharedMessageReference, SharedUser};
use crate::nats::send_nats_message;

#[derive(Serialize)]
pub struct GetMessageRes(Vec<SharedMessage>);

#[derive(Deserialize)]
pub struct GetMessageQuery {
    limit: Option<i32>,
    before: Option<i64>
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
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_channel) => {
            let mut output = vec![];

            let limit = get_message_query.limit.unwrap_or(50);

            let mut messages: Vec<message::Model> = vec![];

            match get_message_query.before {
                None => {
                    messages = Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .limit(limit as u64)
                        .order_by_desc(message::Column::Id)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!");
                }
                Some(before) => {
                    messages = Message::find()
                        .filter(message::Column::ChannelId.eq(requested_channel.id))
                        .limit(limit as u64)
                        .order_by_desc(message::Column::Id)
                        .cursor_by(message::Column::Id)
                        .before(before)
                        .all(&state.conn)
                        .await
                        .expect("Failed to access database!");
                }
            }

            for i in messages {
                let author = User::find_by_id(i.author.unwrap_or(0))
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!");

                let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

                if i.reference_message_id.is_some() {
                    let requested_message = Message::find_by_id(i.reference_message_id.unwrap())
                        .one(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    match requested_message {
                        None => {
                            return StatusCode::BAD_REQUEST.into_response()
                        }
                        // TODO: Prepare this for webhooks
                        Some(requested_message) => {
                            let message_author = User::find_by_id(requested_message.author.unwrap_or(0))
                                .one(&state.conn)
                                .await
                                .expect("Failed to access database!");

                            match message_author {
                                None => {
                                    // webhook?
                                    refed_message = Some((requested_message, None));
                                }
                                Some(message_author) => {
                                    refed_message = Some((requested_message, Some(message_author)));
                                }
                            }
                        }
                    }
                }

                output.push(SharedMessage {
                    attachments: vec![],
                    author: if let Some(author) = author {
                        Option::from(SharedUser {
                            avatar: author.avatar,
                            avatar_decoration: author.avatar_decoration,
                            discriminator: Option::from(author.discriminator),
                            global_name: None,
                            id: author.id.to_string(),
                            public_flags: generate_public_flags(get_user_flags(author.flags)),
                            username: author.username,
                        })
                    } else {
                        None
                    },
                    channel_id: i.channel_id.to_string(),
                    components: vec![],
                    content: i.content,
                    edited_timestamp: None,
                    embeds: vec![],
                    flags: i.flags.unwrap_or(0),
                    id: i.id.to_string(),
                    mention_everyone: i.mention_everyone,
                    mention_roles: None,
                    mentions: None,
                    message_reference: if let Some(message_ref) = refed_message.clone() {
                        Some(SharedMessageReference {
                            channel_id: message_ref.0.channel_id.to_string(),
                            message_id: message_ref.0.id.to_string(),
                        })
                    } else {
                        None
                    },
                    nonce: i.nonce,
                    pinned: false,
                    referenced_message: if let Some(message_ref) = refed_message.clone() {
                        Some(Box::new(
                            SharedMessage {
                                attachments: vec![],
                                author: if let Some(author_ref) = message_ref.1 {
                                    Some(SharedUser {
                                        avatar: author_ref.avatar,
                                        avatar_decoration: author_ref.avatar_decoration,
                                        discriminator: Option::from(author_ref.discriminator),
                                        global_name: None,
                                        id: author_ref.id.to_string(),
                                        public_flags: generate_public_flags(get_user_flags(author_ref.flags)),
                                        username: author_ref.username,
                                    })
                                } else {
                                    None
                                },
                                channel_id: message_ref.0.channel_id.to_string(),
                                components: vec![],
                                content: message_ref.0.content,
                                edited_timestamp: message_ref.0.edited_timestamp.map(|e| e.to_string()),
                                embeds: vec![],
                                flags: message_ref.0.flags.unwrap_or(0),
                                id: message_ref.0.id.to_string(),
                                mention_everyone: message_ref.0.mention_everyone,
                                mention_roles: None,
                                mentions: None,
                                message_reference: None,
                                nonce: message_ref.0.nonce,
                                pinned: message_ref.0.pinned,
                                referenced_message: None,
                                timestamp: message_ref.0.timestamp.to_string(),
                                tts: message_ref.0.tts,
                                _type: message_ref.0.r#type,
                            }
                        ))
                    } else {
                        None
                    } ,
                    timestamp: i.timestamp.to_string(),
                    tts: i.tts,
                    _type: i.r#type,
                });
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
    replied_user: bool
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
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_channel) => {
            let snowflake = Snowflake::default().generate();

            // Does the user have access to this channel?
            if ChannelMember::find_by_id((requested_channel.id, session_context.user.id))
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .is_none() {
                return StatusCode::BAD_REQUEST.into_response()
            }

            // TODO: Guild permission checks

            let broadcast_members: Vec<channel_member::Model> = ChannelMember::find()
                .filter(channel_member::Column::Channel.eq(requested_channel.id))
                .filter(channel_member::Column::User.ne(session_context.user.id))
                .all(&state.conn)
                .await
                .expect("Failed to access database!");

            let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

            if message.message_reference.is_some() {
                let requested_message = Message::find_by_id(message.message_reference.unwrap().message_id.parse::<i64>().unwrap())
                    .one(&state.conn)
                    .await
                    .expect("Failed to access database!");

                match requested_message {
                    None => {
                        return StatusCode::BAD_REQUEST.into_response()
                    }
                    // TODO: Prepare this for webhooks
                    Some(requested_message) => {
                        let message_author = User::find_by_id(requested_message.author.unwrap_or(0))
                            .one(&state.conn)
                            .await
                            .expect("Failed to access database!");

                        match message_author {
                            None => {
                                // webhook?
                                refed_message = Some((requested_message, None));
                            }
                            Some(message_author) => {
                                refed_message = Some((requested_message, Some(message_author)));
                            }
                        }
                    }
                }
            }

            let new_message = message::ActiveModel {
                id: Set(snowflake),
                channel_id: Set(requested_channel.id),
                author: Set(Some(session_context.user.id)),
                content: Set(message.content.clone()),
                timestamp: Set(chrono::Utc::now().naive_utc()),
                tts: Set(message.tts),
                mention_everyone: Set(message.content.contains("@everyone")),
                nonce: Set(Some(message.nonce)),
                r#type: Set({
                    if refed_message.is_some() {
                        MessageTypes::Reply as i32
                    } else {
                        MessageTypes::Default as i32
                    }
                }),
                flags: Set(Some(message.flags)),
                reference_message_id: Set(
                    if let Some(message_ref) = refed_message.clone() {
                        Some(message_ref.0.id)
                    } else {
                        None
                    }
                ),
                reference_channel_id: Set(
                    if let Some(message_ref) = refed_message.clone() {
                        Some(message_ref.0.channel_id)
                    } else {
                        None
                    }
                ),
                pinned: Set(false),
                ..Default::default()
            };

            Message::insert(new_message.clone())
                .exec(&state.conn)
                .await
                .expect("Failed to access database!");

            for i in broadcast_members {
                send_nats_message(
                    &state.nats_client,
                    i.user.to_string(),
                    MessageCreate { id: snowflake },
                ).await;
            }

            (StatusCode::OK, Json(SharedMessage {
                attachments: vec![],
                author: Option::from(SharedUser {
                    avatar: session_context.user.avatar,
                    avatar_decoration: session_context.user.avatar_decoration,
                    discriminator: Option::from(session_context.user.discriminator),
                    global_name: None,
                    id: session_context.user.id.to_string(),
                    public_flags: generate_public_flags(get_user_flags(session_context.user.flags)),
                    username: session_context.user.username,
                }),
                channel_id: new_message.channel_id.unwrap().to_string(),
                components: vec![],
                content: new_message.content.unwrap(),
                edited_timestamp: None,
                embeds: vec![],
                flags: new_message.flags.unwrap().unwrap_or(0),
                id: new_message.id.unwrap().to_string(),
                mention_everyone: new_message.mention_everyone.unwrap(),
                mention_roles: None,
                mentions: None,
                message_reference: if let Some(message_ref) = refed_message.clone() {
                    Some(SharedMessageReference {
                        channel_id: message_ref.0.channel_id.to_string(),
                        message_id: message_ref.0.id.to_string(),
                    })
                } else {
                    None
                },
                nonce: new_message.nonce.unwrap(),
                pinned: false,
                referenced_message: if let Some(message_ref) = refed_message.clone() {
                    Some(Box::new(
                        SharedMessage {
                            attachments: vec![],
                            author: if let Some(author_ref) = message_ref.1 {
                                Some(SharedUser {
                                    avatar: author_ref.avatar,
                                    avatar_decoration: author_ref.avatar_decoration,
                                    discriminator: Option::from(author_ref.discriminator),
                                    global_name: None,
                                    id: author_ref.id.to_string(),
                                    public_flags: generate_public_flags(get_user_flags(author_ref.flags)),
                                    username: author_ref.username,
                                })
                            } else {
                                None
                            },
                            channel_id: message_ref.0.channel_id.to_string(),
                            components: vec![],
                            content: message_ref.0.content,
                            edited_timestamp: message_ref.0.edited_timestamp.map(|e| e.to_string()),
                            embeds: vec![],
                            flags: message_ref.0.flags.unwrap_or(0),
                            id: message_ref.0.id.to_string(),
                            mention_everyone: message_ref.0.mention_everyone,
                            mention_roles: None,
                            mentions: None,
                            message_reference: None,
                            nonce: message_ref.0.nonce,
                            pinned: message_ref.0.pinned,
                            referenced_message: None,
                            timestamp: message_ref.0.timestamp.to_string(),
                            tts: message_ref.0.tts,
                            _type: message_ref.0.r#type,
                        }
                    ))
                } else {
                    None
                } ,
                timestamp: new_message.timestamp.unwrap().to_string(),
                tts: new_message.tts.unwrap(),
                _type: new_message.r#type.unwrap(),
            })).into_response()
        }
    }
}