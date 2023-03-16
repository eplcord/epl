use std::cell::Cell;
use std::collections::HashMap;
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use state::LocalStorage;

pub struct GatewayState<'a> {
    pub(crate) user_id: i64,
    pub(crate) bot: bool,
    pub(crate) compress: bool,
    pub(crate) large_threshold: i8,
    pub(crate) current_shard: i8,
    pub(crate) shard_count: i8,
    pub(crate) intents: i8,
    pub(crate) sender: &'a mut SplitSink<WebSocket, Message>
}

pub static GATEWAY_STATE: LocalStorage<Cell<GatewayState>> = LocalStorage::new();