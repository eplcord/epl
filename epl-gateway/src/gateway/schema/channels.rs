use serde_derive::{Deserialize, Serialize};
use serde_with::{skip_serializing_none};
use epl_common::Stub;
use crate::gateway::schema::SharedUser;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChannelCreate {
    pub flags: i64,
    pub guild_id: Option<String>,
    pub id: String,
    #[serialize_always]
    pub last_message_id: Option<String>,
    pub name: Option<String>,
    pub nsfw: Option<bool>,
    pub parent_id: Option<String>,
    pub permission_overwrites: Option<Vec<Stub>>,
    pub position: Option<i32>,
    pub rate_limit_per_user: Option<i32>,
    pub topic: Option<String>,
    /// Recipients of DM/Group DM
    pub recipients: Option<Vec<SharedUser>>,
    #[serde(rename = "type")]
    pub _type: i32,
    pub version: Option<i32>,
    pub is_spam: Option<bool>
}