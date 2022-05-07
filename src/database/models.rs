use std::time::SystemTime;
use diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub system: bool,
    pub bot: bool,
    pub username: String,
    pub password_hash: String,
    pub discriminator: String,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub banner_colour: Option<String>,
    pub avatar_decoration: Option<String>,
    pub date_of_birth: Option<SystemTime>,
    pub email: String,
    pub phone: Option<String>,
    pub mfa_enabled: bool,
    pub acct_verified: bool,
    pub flags: i32,
    pub nsfw_allowed: Option<bool>,
    pub purchased_flags: Option<i32>,
    pub premium_since: Option<SystemTime>,
    pub premium_flags: Option<i32>,
    pub premium_type: Option<i32>
}

#[derive(Queryable)]
pub struct Session {
    pub id: uuid::Uuid,
    pub user_id: i64,
    pub iat: SystemTime,
    pub exp: SystemTime,
}