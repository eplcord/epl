use serde_derive::{Deserialize, Serialize};
use crate::gateway::schema::SharedUser;

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipAdd {
    pub id: String,
    pub nickname: Option<String>,
    pub should_notify: bool,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i32,
    pub user: SharedUser
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipRemove {
    pub id: String,
    pub nickname: Option<String>,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i32,
}