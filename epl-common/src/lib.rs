use std::net::IpAddr;
use maxminddb::geoip2::{City};
use maxminddb::Reader;
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use crate::options::{EplOptions, Options};

pub mod options;
pub mod rustflake;
pub mod database;
pub mod flags;

static GEOIP: Lazy<Reader<Vec<u8>>> = Lazy::new(|| {
    Reader::open_readfile(EplOptions::get().maxminddb).expect("Failed to open maxmind database!")
});

pub fn gen_token() -> String {
    blake3::hash(Alphanumeric.sample_string(&mut rand::thread_rng(), 32).as_bytes()).to_hex().to_string()
}

pub fn gen_session_id() -> String {
    blake3::hash(Alphanumeric.sample_string(&mut rand::thread_rng(), 16).as_bytes()).to_hex().to_string()
}

pub fn get_location_from_ip(ip: IpAddr) -> String {
    if ip.to_string().eq("127.0.0.1") {
        return String::from("🤨");
    }

    let result: City = GEOIP.lookup(ip).expect("Failed to look up IP");

    format!("{}, {} ({})",
            result.city.unwrap().names.unwrap().first_entry().unwrap().get(),
            result.country.unwrap().names.unwrap().first_entry().unwrap().get(),
            ip
    )
}