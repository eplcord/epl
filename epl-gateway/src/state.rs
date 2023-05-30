use std::net::IpAddr;
use std::str::FromStr;
use async_nats::Subscriber;
use axum::extract::ws::WebSocket;

#[derive(Debug, Eq, PartialEq)]
pub struct ParseCompressionTypeError;

impl FromStr for CompressionType {
    type Err = ParseCompressionTypeError;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        match t {
            "zlib" => Ok(Self::Zlib),
            "zlib-stream" => Ok(Self::ZlibStreams),
            "zstd-stream" => Ok(Self::ZstdStreams),
            _ => Err(ParseCompressionTypeError{})
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
            _ => Err(ParseEncodingTypeError{})
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum CompressionType {
    Zlib,
    ZlibStreams,
    ZstdStreams
}

#[derive(Eq, PartialEq, Clone)]
pub enum EncodingType {
    Json,
    Etf
}

pub struct GatewayState {
    pub(crate) user_id: Option<i64>,
    pub(crate) bot: Option<bool>,
    pub(crate) large_threshold: Option<i8>,
    pub(crate) current_shard: Option<i8>,
    pub(crate) shard_count: Option<i8>,
    pub(crate) intents: Option<i8>,
    pub(crate) compression: Option<CompressionType>,
    pub(crate) encoding: EncodingType,
}


// pub static GATEWAY_STATE: LocalStorage<Arc<Mutex<Option<GatewayState>>>> = LocalStorage::new();
//
// pub static SOCKET: LocalStorage<Arc<Mutex<Option<WebSocket>>>> = LocalStorage::new();
//
// pub static NATS: LocalStorage<Arc<Mutex<Option<async_nats::Client>>>> = LocalStorage::new();
//
// pub static NATS_SUBSCRIPTIONS: LocalStorage<Arc<Mutex<Option<Vec<Subscriber>>>>> = LocalStorage::new();

#[derive(Default)]
pub struct ThreadData {
    pub gateway_state: Option<GatewayState>,
    pub socket: Option<WebSocket>,
    pub nats: Option<async_nats::Client>,
    pub nats_subscriptions: Option<Vec<Subscriber>>,
    pub session_ip: Option<IpAddr>,
}