use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Ready {
    pub version: i32,
    pub users: Vec<Stub>,
    pub user_settings_proto: String,
    pub user_guild_settings: UserGuildSettings,
    pub user: User,
    pub tutorial: Tutorial,
    pub sessions: Vec<Session>,
    pub session_type: String,
    pub session_id: String,
    pub resume_gateway_url: String,
    pub relationships: Vec<Stub>,
    pub read_state: ReadState,
    pub private_channels: Vec<Stub>,
    pub merged_members: Vec<Stub>,
    pub guilds: Vec<Stub>,
    pub guild_join_requests: Vec<Stub>,
    pub guild_experiments: Vec<Stub>,
    pub geo_ordered_rtc_regions: Vec<String>,
    pub friend_suggestion_count: i32,
    pub experiments: Vec<Stub>,
    pub country_code: String,
    pub consents: Consents,
    pub connected_accounts: Vec<Stub>,
    pub auth_session_id_hash: String,
    pub api_code_version: i32,
    pub analytics_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserGuildSettings {
    pub version: i32,
    pub partial: bool,
    pub entries: Vec<Stub>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub verified: bool,
    pub username: String,
    pub purchased_flags: i32,
    pub premium_type: i32,
    pub premium: bool,
    pub phone: Option<String>,
    pub nsfw_allowed: bool,
    pub mobile: bool,
    pub mfa_enabled: bool,
    pub id: String,
    pub global_name: Option<String>,
    pub flags: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub discriminator: String,
    pub desktop: bool,
    pub bio: String,
    pub banner_color: Option<String>,
    pub banner: Option<String>,
    pub avatar_decoration: Option<String>,
    pub avatar: Option<String>,
    pub accent_color: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Tutorial {
    pub indicators_suppressed: bool,
    pub indicators_confirmed: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub status: String,
    pub session_id: String,
    pub client_info: SessionClientInfo,
    pub activities: Vec<Stub>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionClientInfo {
    pub version: i32,
    pub os: String,
    pub client: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadState {
    pub version: i32,
    pub partial: bool,
    pub entries: Vec<ReadStateEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct ReadStateEntry {
    pub read_state_type: i32,
    pub last_acked_id: String,
    pub id: String,
    pub badge_count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Consents {
    pub personalization: ConsentsEntry,
}

#[derive(Serialize, Deserialize)]
pub struct ConsentsEntry {
    pub consented: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ReadySupplemental {
    pub disclose: Vec<String>,
    pub guilds: Vec<Stub>,
    pub lazy_private_channels: Vec<Stub>,
    pub merged_members: Vec<Stub>,
    pub merged_presences: MergedPresences,
}

#[derive(Serialize, Deserialize)]
pub struct MergedPresences {
    pub friends: Vec<Stub>,
    pub guilds: Vec<Stub>,
}

#[derive(Serialize, Deserialize)]
pub struct Stub {}
