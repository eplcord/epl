use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Presence {
    status: Option<String>,
    since: Option<i32>,
    activities: Option<Vec<Activity>>,
    client_status: Option<ClientStatus>,
    afk: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientStatus {
    #[serde(rename = "desktop")]
    Desktop(String),
    #[serde(rename = "mobile")]
    Mobile(String),
    #[serde(rename = "web")]
    Web(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Activity {}
