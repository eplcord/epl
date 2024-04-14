use serde_derive::{Deserialize, Serialize};
use crate::database::entities::user;
use crate::flags::{generate_public_flags, get_user_flags};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String,
}

pub fn generate_user_struct(user: user::Model) -> User {
    User {
        avatar: user.avatar,
        avatar_decoration: user.avatar_decoration,
        discriminator: Option::from(user.discriminator),
        global_name: user.display_name.clone(),
        id: user.id.to_string(),
        public_flags: generate_public_flags(get_user_flags(user.flags)),
        username: user.username,
    }
}