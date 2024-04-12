use sea_orm::{DatabaseConnection, EntityTrait};
use crate::gateway::schema::{generated_user_struct, SharedUser};
use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use epl_common::database::entities::{embed, file, message, user};
use epl_common::database::entities::prelude::{Message, User};
use epl_common::options::{EplOptions, Options};

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageDelete {
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub id: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageReference {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SharedMessage {
    pub attachments: Vec<SharedAttachment>,
    pub author: Option<SharedUser>,
    pub channel_id: String,
    pub components: Vec<Stub>,
    pub content: String,
    pub edited_timestamp: Option<String>,
    pub embeds: Vec<Value>,
    pub flags: i32,
    pub id: String,
    pub mention_everyone: bool,
    pub mention_roles: Option<Stub>,
    pub mentions: Option<Vec<SharedUser>>,
    pub message_reference: Option<MessageReference>,
    pub nonce: Option<String>,
    pub pinned: bool,
    pub referenced_message: Option<Box<SharedMessage>>,
    pub timestamp: String,
    pub tts: bool,
    #[serde(rename = "type")]
    pub _type: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
pub struct SharedAttachment {
    id: String,
    filename: String,
    content_type: String,
    placeholder: Option<String>,
    placeholder_version: Option<u8>,
    content_scan_version: Option<u8>,
    url: String,
    proxy_url: String,
    size: i64,
    height: Option<i64>,
    width: Option<i64>
}

pub fn generate_message_struct(
    message: message::Model,
    author: Option<user::Model>,
    ref_message: Option<(message::Model, Option<user::Model>)>,
    mentions: Vec<user::Model>,
    pinned: bool,
    embeds: Vec<embed::Model>,
    attachments: Vec<file::Model>
) -> SharedMessage {
    let options = EplOptions::get();

    SharedMessage {
        attachments: attachments.iter().map(|x| {
            let attachment_url = format!("{}://{}/attachments/{}/{}/{}",
                                         if options.require_ssl { "https" } else { "http" },
                                         options.cdn_url,
                                         message.channel_id.clone(),
                                         x.id,
                                         x.name.clone()
            );

            SharedAttachment {
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
        author: author.map(generated_user_struct),
        channel_id: message.channel_id.to_string(),
        components: vec![],
        content: message.content,
        edited_timestamp: message.edited_timestamp.map(|e| e.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string()),
        embeds: embeds.iter().map(|x| x.content.clone()).collect(),
        flags: message.flags.unwrap_or(0),
        id: message.id.to_string(),
        mention_everyone: message.mention_everyone,
        mention_roles: None,
        mentions: Some(mentions.into_iter().map(generated_user_struct).collect()),
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
            Some(Box::new(generate_message_struct(ref_message.0, ref_message.1, None, vec![], false, vec![], vec![])))
        } else {
            None
        },
        timestamp: message.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        tts: message.tts,
        _type: message.r#type,
    }
}

pub async fn generate_refed_message(conn: &DatabaseConnection, id: i64) -> Option<(message::Model, Option<user::Model>)> {
    let requested_message = Message::find_by_id(id)
        .one(conn)
        .await
        .expect("Failed to access database!");

    match requested_message {
        None => None,
        // TODO: Prepare this for webhooks
        Some(requested_message) => {
            let message_author =
                User::find_by_id(requested_message.author.unwrap_or(0))
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