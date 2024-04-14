use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageDelete {
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub id: String
}