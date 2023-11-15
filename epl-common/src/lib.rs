use crate::options::{EplOptions, Options};
use maxminddb::geoip2::City;
use maxminddb::Reader;
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

static GEOIP: Lazy<Reader<Vec<u8>>> = Lazy::new(|| {
    Reader::open_readfile(EplOptions::get().maxminddb).expect("Failed to open maxmind database!")
});

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

    let result: City = GEOIP.lookup(ip).expect("Failed to look up IP");

    format!(
        "{}, {} ({})",
        result.city.unwrap().names.unwrap().get("en").unwrap(),
        result.country.unwrap().names.unwrap().get("en").unwrap(),
        ip
    )
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
