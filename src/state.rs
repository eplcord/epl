use std::cell::Cell;
use std::sync::Arc;
use axum::extract::ws::WebSocket;
use state::LocalStorage;
use tokio::sync::Mutex;

pub struct GatewayState {
    pub(crate) user_id: i64,
    pub(crate) bot: bool,
    pub(crate) compress: bool,
    pub(crate) large_threshold: i8,
    pub(crate) current_shard: i8,
    pub(crate) shard_count: i8,
    pub(crate) intents: i8,
}

pub struct WebSocketWrapper {
    pub inner: WebSocket,
}

pub static GATEWAY_STATE: LocalStorage<Arc<Mutex<GatewayState>>> = LocalStorage::new();

pub static SOCKET: LocalStorage<Arc<Mutex<WebSocketWrapper>>> = LocalStorage::new();