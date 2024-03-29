use crate::gateway::schema::SharedUser;
use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChannelCreate {
    pub flags: i64,
    pub guild_id: Option<String>,
    pub id: String,
    #[serialize_always]
    pub last_message_id: Option<String>,
    pub name: Option<String>,
    #[serialize_always]
    pub icon: Option<String>,
    pub nsfw: Option<bool>,
    pub parent_id: Option<String>,
    pub permission_overwrites: Option<Vec<Stub>>,
    pub position: Option<i32>,
    pub rate_limit_per_user: Option<i32>,
    pub topic: Option<String>,
    pub owner_id: Option<String>,
    /// Recipients of DM/Group DM
    pub recipients: Option<Vec<SharedUser>>,
    #[serde(rename = "type")]
    pub _type: i32,
    pub version: Option<i32>,
    pub is_spam: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelDelete {
    pub flags: i64,
    pub guild_id: Option<String>,
    pub id: String,
    #[serialize_always]
    pub last_message_id: Option<String>,
    pub name: Option<String>,
    #[serialize_always]
    pub icon: Option<String>,
    pub owner_id: Option<String>,
    #[serde(rename = "type")]
    pub _type: i32,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelRecipientAdd {
    pub channel_id: String,
    pub user: SharedUser,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelRecipientRemove {
    pub channel_id: String,
    pub user: SharedUser,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelPinsUpdate {
    #[serialize_always]
    pub last_pin_timestamp: Option<String>,
    pub channel_id: String,
    pub guild_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelPinsAck {
    pub timestamp: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub version: i64
}