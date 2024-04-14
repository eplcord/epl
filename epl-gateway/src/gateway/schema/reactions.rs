use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use epl_common::schema::v9::message::Emoji;
use epl_common::Stub;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct MessageReactionAdd {
    pub user_id: String,
    #[serde(rename = "type")]
    pub _type: i32,
    pub message_id: String,
    pub message_author_id: Option<String>,
    // TODO: Implement when guilds are a thing
    pub member: Option<Stub>,
    pub emoji: Emoji,
    pub channel_id: String,
    pub burst: bool,
    pub guild_id: Option<String>
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct MessageReactionRemove {
    pub user_id: String,
    #[serde(rename = "type")]
    pub _type: i32,
    pub message_id: String,
    pub emoji: Emoji,
    pub channel_id: String,
    pub burst: bool,
    pub guild_id: Option<String>
}