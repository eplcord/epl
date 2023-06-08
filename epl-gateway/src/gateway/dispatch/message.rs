use crate::gateway::dispatch::{assemble_dispatch, send_message, DispatchTypes};
use crate::gateway::schema::message::{MessageCreate, MessageReference};
use crate::state::ThreadData;
use crate::AppState;
use epl_common::database::entities::prelude::{Message, User};

use crate::gateway::schema::SharedUser;
use epl_common::database::entities::{message, user};
use epl_common::flags::{generate_public_flags, get_user_flags};
use sea_orm::prelude::*;

pub async fn dispatch_message_create(thread_data: &mut ThreadData, state: &AppState, id: i64) {
    let message = Message::find_by_id(id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Failed to get message requested by NATS!");

    let message_author = User::find_by_id(message.author.unwrap_or(0))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

    if message.reference_message_id.is_some() {
        let requested_message = Message::find_by_id(message.reference_message_id.unwrap())
            .one(&state.conn)
            .await
            .expect("Failed to access database!");

        match requested_message {
            None => {
                refed_message = None;
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

    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::MessageCreate(MessageCreate {
            attachments: vec![],
            author: message_author.map(|message_author| SharedUser {
                avatar: message_author.avatar,
                avatar_decoration: message_author.avatar_decoration,
                discriminator: Option::from(message_author.discriminator),
                global_name: None,
                id: message_author.id.to_string(),
                public_flags: generate_public_flags(get_user_flags(message_author.flags)),
                username: message_author.username,
            }),
            channel_id: message.channel_id.to_string(),
            components: vec![],
            content: message.content,
            edited_timestamp: message.edited_timestamp.map(|e| e.to_string()),
            embeds: vec![],
            flags: message.flags.unwrap_or(0),
            id: message.id.to_string(),
            mention_everyone: message.mention_everyone,
            mention_roles: None,
            mentions: None,
            message_reference: if let Some(message_ref) = refed_message.clone() {
                Some(MessageReference {
                    channel_id: message_ref.0.channel_id.to_string(),
                    message_id: message_ref.0.id.to_string(),
                })
            } else {
                None
            },
            nonce: message.nonce,
            pinned: message.pinned,
            referenced_message: if let Some(message_ref) = refed_message.clone() {
                Some(Box::new(MessageCreate {
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
                }))
            } else {
                None
            },
            timestamp: message.timestamp.to_string(),
            tts: message.tts,
            _type: message.r#type,
        })),
    )
    .await;
}
