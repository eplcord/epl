use crate::options::{EplOptions, Options};
use maxminddb::geoip2::City;
use maxminddb::{MaxMindDBError, Reader};
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;

pub mod channels;
pub mod database;
pub mod flags;
pub mod messages;
pub mod nats;
pub mod nodeinfo;
pub mod options;
pub mod rustflake;
pub mod permissions;
pub mod relationship;
mod protobufs;
pub mod tenor;

static GEOIP: Lazy<Reader<Vec<u8>>> = Lazy::new(|| {
    Reader::open_readfile(EplOptions::get().maxminddb).expect("Failed to open maxmind database!")
});

pub static USER_MENTION_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"<@!?(\d+)>").unwrap());

pub static URL_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap());

pub fn gen_token() -> String {
    blake3::hash(
        Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .as_bytes(),
    )
    .to_hex()
    .to_string()
}

pub fn gen_session_id() -> String {
    blake3::hash(
        Alphanumeric
            .sample_string(&mut rand::thread_rng(), 16)
            .as_bytes(),
    )
    .to_hex()
    .to_string()
}

pub fn get_location_from_ip(ip: IpAddr) -> String {
    if ip.to_string().eq("127.0.0.1") {
        return String::from("🤨");
    }

    let result: Result<City, MaxMindDBError> = GEOIP.lookup(ip);
    
    match result {
        Ok(result) => {
            format!(
                "{}, {} ({})",
                result.city.unwrap().names.unwrap().get("en").unwrap(),
                result.country.unwrap().names.unwrap().get("en").unwrap(),
                ip
            )
        }
        Err(_) => {
            String::from("Locator service not functioning!")
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stub {}

#[derive(Serialize, Deserialize, Debug)]
#[repr(i32)]
pub enum RelationshipType {
    Friend = 1,
    Blocked = 2,
    Incoming = 3,
    Outgoing = 4,
}

#[derive(Serialize, Deserialize, Clone)]
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
