use crate::gateway::schema::presence::Presence;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Identify {
    pub token: String,
    pub capabilities: Option<i32>,
    pub properties: Option<Properties>,
    pub presence: Option<Presence>,
    pub compress: Option<bool>,
    pub client_state: Option<ClientState>
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    pub browser: Option<String>,
    pub client_build_number: Option<i32>,
    pub client_event_source: Option<String>,
    pub client_version: Option<String>,
    pub distro: Option<String>,
    pub os: Option<String>,
    pub os_arch: Option<String>,
    pub system_locale: Option<String>,
    pub window_manager: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct ClientState {
    pub highest_last_message_id: Option<String>,
    pub read_state_version: Option<i32>,
    pub user_guild_settings_version: Option<i32>,
    pub user_settings_version: Option<i32>,
    pub guild_hashes: Option<GuildHashes>
}

#[derive(Serialize, Deserialize)]
pub struct GuildHashes {
    // i forgor 💀
}