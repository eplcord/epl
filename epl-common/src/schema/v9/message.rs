use std::collections::HashSet;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use serde_json::Value;
use crate::database::entities::{embed, file, message, reaction, user};
use crate::database::entities::prelude::Reaction as ReactionEntity;
use crate::options::{EplOptions, Options};
use crate::schema::v9::user::{generate_user_struct, User};
use crate::Stub;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub attachments: Vec<Attachment>,
    pub author: Option<User>,
    pub channel_id: String,
    pub components: Vec<Stub>,
    pub content: String,
    pub edited_timestamp: Option<String>,
    pub embeds: Vec<Value>,
    pub flags: i32,
    pub id: String,
    pub mention_everyone: bool,
    pub mention_roles: Option<Stub>,
    pub mentions: Option<Vec<User>>,
    pub message_reference: Option<MessageReference>,
    pub nonce: Option<String>,
    pub pinned: bool,
    pub referenced_message: Option<Box<Message>>,
    pub timestamp: String,
    pub tts: bool,
    #[serde(rename = "type")]
    pub _type: i32,
    pub reactions: Vec<Reaction>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageReference {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub placeholder: Option<String>,
    pub placeholder_version: Option<u8>,
    pub content_scan_version: Option<u8>,
    pub url: String,
    pub proxy_url: String,
    pub size: i64,
    pub height: Option<i64>,
    pub width: Option<i64>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Reaction {
    burst_colors: Vec<String>,
    burst_count: i64,
    burst_me: bool,
    count: i64,
    me: bool,
    me_burst: bool,
    emoji: Emoji,
    count_details: CountDetails
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CountDetails {
    burst: i64,
    normal: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Emoji {
    pub id: Option<String>,
    pub name: String,
}

pub fn generate_message_struct(
    message: message::Model,
    author: Option<user::Model>,
    ref_message: Option<(message::Model, Option<user::Model>)>,
    mentions: Vec<user::Model>,
    pinned: bool,
    embeds: Vec<embed::Model>,
    attachments: Vec<file::Model>,
    reactions: Vec<Reaction>
) -> Message {
    let options = EplOptions::get();

    Message {
        attachments: attachments.iter().map(|x| {
            let attachment_url = format!("{}://{}/attachments/{}/{}/{}",
                                         if options.require_ssl { "https" } else { "http" },
                                         options.cdn_url,
                                         message.channel_id.clone(),
                                         x.id,
                                         x.name.clone()
            );

            Attachment {
                id: x.id.to_string(),
                filename: x.name.clone(),
                content_type: x.content_type.clone().unwrap_or("application/octet-stream".to_string()),
                placeholder: None,
                placeholder_version: None,
                content_scan_version: None,
                url: attachment_url.clone(),
                proxy_url: attachment_url,
                size: x.size,
                height: x.height,
                width: x.width,
            }
        }).collect(),
        author: author.map(generate_user_struct),
        channel_id: message.channel_id.to_string(),
        components: vec![],
        content: message.content,
        edited_timestamp: message.edited_timestamp.map(|e| e.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string()),
        embeds: embeds.iter().map(|x| x.content.clone()).collect(),
        flags: message.flags.unwrap_or(0),
        id: message.id.to_string(),
        mention_everyone: message.mention_everyone,
        mention_roles: None,
        mentions: Some(mentions.into_iter().map(generate_user_struct).collect()),
        message_reference: if let Some(ref_message) = ref_message.clone() {
            Some(MessageReference {
                channel_id: ref_message.0.channel_id.to_string(),
                message_id: ref_message.0.id.to_string(),
            })
        } else {
            None
        },
        nonce: message.nonce,
        pinned,
        referenced_message: if let Some(ref_message) = ref_message {
            Some(Box::new(generate_message_struct(ref_message.0, ref_message.1, None, vec![], false, vec![], vec![], vec![])))
        } else {
            None
        },
        timestamp: message.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        tts: message.tts,
        _type: message.r#type,
        reactions,
    }
}

pub async fn generate_refed_message(conn: &DatabaseConnection, id: i64) -> Option<(message::Model, Option<user::Model>)> {
    let requested_message = crate::database::entities::prelude::Message::find_by_id(id)
        .one(conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => None,
        // TODO: Prepare this for webhooks
        Some(requested_message) => {
            let message_author =
                crate::database::entities::prelude::User::find_by_id(requested_message.author.unwrap_or(0))
                    .one(conn)
                    .await
                    .expect("Failed to access database!");

            match message_author {
                None => {
                    // webhook?
                    Some((requested_message, None))
                }
                Some(message_author) => {
                    Some((requested_message, Some(message_author)))
                }
            }
        }
    }
}

pub async fn generate_reactions(
    conn: &DatabaseConnection,
    message: &message::Model,
    current_user: &i64,
) -> Vec<Reaction> {
    // TODO: guild emojis when that exists
    let mut final_reactions = vec![];

    let mut visited_emojis: HashSet<String> = HashSet::new();

    let reactions = message.find_related(ReactionEntity)
        .filter(reaction::Column::Burst.eq(false))
        .all(conn)
        .await
        .expect("Failed to access database!");

    let burst_reactions = message.find_related(ReactionEntity)
        .filter(reaction::Column::Burst.eq(true))
        .all(conn)
        .await
        .expect("Failed to access database!");

    for i in reactions.clone() {
        visited_emojis.insert(i.emoji);
    }

    for i in burst_reactions.clone() {
        visited_emojis.insert(i.emoji);
    }

    for i in visited_emojis {
        let burst_reactions_iter: Vec<i64> = burst_reactions.iter().filter(|x| x.burst && x.emoji == i).map(|x| x.user).collect();
        let reactions_iter: Vec<i64> = reactions.iter().filter(|x| !x.burst && x.emoji == i).map(|x| x.user).collect();

        final_reactions.push(
            Reaction {
                // TODO: figure out how to calculate these
                burst_colors: vec![],
                burst_count: burst_reactions_iter.len() as i64,
                burst_me: burst_reactions_iter.contains(current_user),
                count: reactions_iter.len() as i64,
                me:reactions_iter.contains(current_user),
                me_burst: burst_reactions_iter.contains(current_user),
                // TODO: guild emojis when that exists
                emoji: Emoji {
                    id: None,
                    name: i
                },
                count_details: CountDetails {
                    burst: burst_reactions_iter.len() as i64,
                    normal: reactions_iter.len() as i64
                },
            }
        )
    }

    final_reactions
}
