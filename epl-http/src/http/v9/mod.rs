use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};

pub(crate) mod errors;
pub(crate) mod routes;

#[derive(Serialize)]
pub struct SharedUser {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String,
}

#[derive(Serialize)]
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
