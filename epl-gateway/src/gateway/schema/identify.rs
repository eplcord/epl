use crate::gateway::schema::presence::Presence;
use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Identify {
    pub token: String,
    pub capabilities: Option<u64>,
    pub properties: Option<Properties>,
    pub presence: Option<Presence>,
    pub compress: Option<bool>,
    pub client_state: Option<ClientState>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Properties {
    pub browser: Option<String>,
    pub browser_user_agent: Option<String>,
    pub browser_version: Option<String>,
    pub client_build_number: Option<u64>,
    pub client_event_source: Option<String>,
    pub client_version: Option<String>,
    // TODO: Research this more and make sure its actually an i32
    pub native_build_number: Option<i32>,
    pub distro: Option<String>,
    pub os: Option<String>,
    pub os_arch: Option<String>,
    pub os_version: Option<String>,
    pub release_channel: Option<String>,
    pub system_locale: Option<String>,
    pub window_manager: Option<String>,
    pub device: Option<String>,
    pub device_vendor_id: Option<String>,
    pub design_id: Option<i32>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientState {
    pub api_code_version: Option<i32>,
    pub guild_versions: Option<Stub>,
    pub highest_last_message_id: Option<String>,
    pub private_channels_versions: Option<String>,
    pub read_state_version: Option<u64>,
    pub user_guild_settings_version: Option<u64>,
    pub user_settings_version: Option<u64>,
    pub guild_hashes: Option<Stub>,
}
