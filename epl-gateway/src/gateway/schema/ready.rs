use epl_common::{Stub, User};
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
pub struct Ready {
    pub version: i32,
    pub users: Vec<OtherUser>,
    pub user_settings_proto: String,
    pub user_guild_settings: UserGuildSettings,
    pub user: User,
    pub tutorial: Tutorial,
    pub sessions: Vec<Session>,
    pub session_type: String,
    pub session_id: String,
    pub resume_gateway_url: String,
    pub relationships: Vec<RelationshipReady>,
    pub read_state: ReadState,
    pub private_channels: Vec<PrivateChannel>,
    pub merged_members: Vec<Stub>,
    pub guilds: Vec<Stub>,
    pub guild_join_requests: Vec<Stub>,
    pub guild_experiments: Vec<Stub>,
    pub geo_ordered_rtc_regions: Vec<String>,
    pub friend_suggestion_count: i32,
    pub experiments: Vec<[i32; 8]>,
    pub country_code: String,
    pub consents: Consents,
    pub connected_accounts: Vec<Stub>,
    pub auth_session_id_hash: String,
    pub api_code_version: i32,
    pub analytics_token: String,
    pub notification_settings: Option<Stub>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserGuildSettings {
    pub version: i32,
    pub partial: bool,
    pub entries: Vec<Stub>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tutorial {
    pub indicators_suppressed: bool,
    pub indicators_confirmed: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub status: String,
    pub session_id: String,
    pub client_info: SessionClientInfo,
    pub activities: Vec<Stub>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionClientInfo {
    pub version: i32,
    pub os: String,
    pub client: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReadState {
    pub version: i32,
    pub partial: bool,
    pub entries: Vec<ReadStateEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReadStateEntry {
    pub read_state_type: i32,
    pub last_acked_id: String,
    pub id: String,
    pub badge_count: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Consents {
    pub personalization: ConsentsEntry,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConsentsEntry {
    pub consented: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReadySupplemental {
    pub disclose: Vec<String>,
    pub guilds: Vec<Stub>,
    pub lazy_private_channels: Vec<Stub>,
    pub merged_members: Vec<Stub>,
    pub merged_presences: MergedPresences,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MergedPresences {
    pub friends: Vec<Stub>,
    pub guilds: Vec<Stub>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationshipReady {
    pub user_id: String,
    #[serde(rename = "type")]
    pub _type: i32,
    pub since: String,
    pub nickname: Option<String>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OtherUser {
    pub username: String,
    pub public_flags: i64,
    pub id: String,
    pub global_name: Option<String>,
    pub discriminator: Option<String>,
    pub bot: bool,
    pub avatar_decoration: Option<String>,
    pub avatar: Option<String>,
}

/// These are just DMs/Group DMs
#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateChannel {
    #[serde(rename = "type")]
    pub _type: i32,
    pub recipient_ids: Vec<String>,
    pub last_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_spam: Option<bool>,
    pub id: String,
    pub flags: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>
}
