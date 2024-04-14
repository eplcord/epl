use serde_derive::{Deserialize, Serialize};
use epl_common::schema::v9;

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipAdd {
    pub id: String,
    pub nickname: Option<String>,
    pub should_notify: bool,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i32,
    pub user: v9::user::User,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipRemove {
    pub id: String,
    pub nickname: Option<String>,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i32,
}
