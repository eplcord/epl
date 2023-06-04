use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipAdd {
    pub id: String,
    pub nickname: Option<String>,
    pub should_notify: bool,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i8,
    pub user: User
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipRemove {
    pub id: String,
    pub nickname: Option<String>,
    pub since: String,
    #[serde(rename = "type")]
    pub _type: i8,
}