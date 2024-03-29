use std::collections::HashMap;
use async_nats::Subscriber;
use axum_tungstenite::WebSocket;
use epl_common::rustflake::Snowflake;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub struct ParseCompressionTypeError;

impl FromStr for CompressionType {
    type Err = ParseCompressionTypeError;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        match t {
            "zlib" => Ok(Self::Zlib),
            "zlib-stream" => Ok(Self::ZlibStreams),
            "zstd-stream" => Ok(Self::ZstdStreams),
            _ => Err(ParseCompressionTypeError {}),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseEncodingTypeError;

impl FromStr for EncodingType {
    type Err = ParseEncodingTypeError;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        match t {
            "etf" => Ok(Self::Etf),
            "json" => Ok(Self::Json),
            _ => Err(ParseEncodingTypeError {}),
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum CompressionType {
    Zlib,
    ZlibStreams,
    ZstdStreams,
}

#[derive(Eq, PartialEq, Clone)]
pub enum EncodingType {
    Json,
    Etf,
}

#[derive(Eq, PartialEq, Clone)]
pub struct GatewayState {
    pub(crate) gateway_session_id: i64,
    pub(crate) user_id: Option<i64>,
    pub(crate) session_id: Option<String>,
    pub(crate) bot: Option<bool>,
    pub(crate) large_threshold: Option<i8>,
    pub(crate) current_shard: Option<i8>,
    pub(crate) shard_count: Option<i8>,
    pub(crate) intents: Option<i8>,
    pub(crate) compression: Option<CompressionType>,
    pub(crate) encoding: EncodingType,
    pub(crate) sequence: i64
}

pub struct ThreadData {
    pub gateway_state: GatewayState,
    pub socket: WebSocket,
    pub nats: async_nats::Client,
    pub nats_subscriptions: HashMap<String, Subscriber>,
    pub session_ip: IpAddr,
    pub snowflake_factory: Snowflake,
}
