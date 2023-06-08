use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait};
use epl_common::database::entities::{message, user};
use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::prelude::{Message, User};
use epl_common::flags::{generate_public_flags, get_user_flags};

pub(crate) mod errors;
pub(crate) mod routes;

#[derive(Serialize, Deserialize)]
pub struct SharedUser {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct SharedMessage {
    attachments: Vec<Stub>,
    author: Option<SharedUser>,
    channel_id: String,
    components: Vec<Stub>,
    content: String,
    edited_timestamp: Option<String>,
    embeds: Vec<Stub>,
    flags: i32,
    id: String,
    mention_everyone: bool,
    mention_roles: Option<Stub>,
    mentions: Option<Vec<String>>,
    message_reference: Option<SharedMessageReference>,
    nonce: Option<String>,
    pinned: bool,
    referenced_message: Option<Box<SharedMessage>>,
    timestamp: String,
    tts: bool,
    #[serde(rename = "type")]
    _type: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SharedMessageReference {
    channel_id: String,
    message_id: String,
}

pub fn generated_user_struct(user: user::Model) -> SharedUser {
    SharedUser {
        avatar: user.avatar,
        avatar_decoration: user.avatar_decoration,
        discriminator: Option::from(user.discriminator),
        global_name: None,
        id: user.id.to_string(),
        public_flags: generate_public_flags(get_user_flags(user.flags)),
        username: user.username,
    }
}

pub fn generate_message_struct(
    message: message::Model,
    author: Option<user::Model>,
    ref_message: Option<(message::Model, Option<user::Model>)>,
) -> SharedMessage {
    SharedMessage {
        attachments: vec![],
        author: author.map(generated_user_struct),
        channel_id: message.channel_id.to_string(),
        components: vec![],
        content: message.content,
        edited_timestamp: message.edited_timestamp.map(|e| e.and_local_timezone(Utc).unwrap().to_string()),
        embeds: vec![],
        flags: message.flags.unwrap_or(0),
        id: message.id.to_string(),
        mention_everyone: message.mention_everyone,
        mention_roles: None,
        mentions: None,
        message_reference: if let Some(ref_message) = ref_message.clone() {
            Some(SharedMessageReference {
                channel_id: ref_message.0.channel_id.to_string(),
                message_id: ref_message.0.id.to_string(),
            })
        } else {
            None
        },
        nonce: message.nonce,
        pinned: message.pinned,
        referenced_message: if let Some(ref_message) = ref_message {
            Some(Box::new(generate_message_struct(ref_message.0, ref_message.1, None)))
        } else {
            None
        },
        timestamp: message.timestamp.and_local_timezone(Utc).unwrap().to_string(),
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