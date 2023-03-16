use rand::distributions::{Alphanumeric, DistString};

pub mod rustflake;

pub fn gen_token() -> String {
    blake3::hash(Alphanumeric.sample_string(&mut rand::thread_rng(), 32).as_bytes()).to_hex().to_string()
}