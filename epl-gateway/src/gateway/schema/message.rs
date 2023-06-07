use serde_derive::{Deserialize, Serialize};
use epl_common::Stub;
use crate::gateway::schema::SharedUser;

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageReference {
    pub channel_id: String,
    pub message_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageCreate {
    pub attachments: Vec<Stub>,
    pub author: Option<SharedUser>,
    pub channel_id: String,
    pub components: Vec<Stub>,
    pub content: String,
    pub edited_timestamp: Option<String>,
    pub embeds: Vec<Stub>,
    pub flags: i32,
    pub id: String,
    pub mention_everyone: bool,
    pub mention_roles: Option<Stub>,
    pub mentions: Option<Vec<String>>,
    pub message_reference: Option<MessageReference>,
    pub nonce: Option<String>,
    pub pinned: bool,
    pub referenced_message: Option<Box<MessageCreate>>,
    pub timestamp: String,
    pub tts: bool,
    #[serde(rename = "type")]
    pub _type: i32,
}