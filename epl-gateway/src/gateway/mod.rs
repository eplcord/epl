use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::ws::Message::{Close, Ping, Pong, Text},
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use axum::extract::Query;
use futures::StreamExt;
use serde::{de, Deserialize, Deserializer};

use crate::AppState;
use tracing::{debug, info};
use epl_common::options::{EplOptions, Options};
use crate::gateway::dispatch::send_message;

use crate::gateway::handle::handle_op;
use crate::gateway::schema::hello::Hello;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};
use crate::gateway::schema::GatewayMessage;
use crate::state::{SOCKET, NATS, NATS_SUBSCRIPTIONS, GatewayState, GATEWAY_STATE, EncodingType, CompressionType};

mod dispatch;
mod handle;
mod schema;

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Deserialize)]
pub struct Params {
    pub encoding: String,
    pub v: i32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub compress: Option<String>
}

pub async fn gateway(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<AppState>,
    Query(params): Query<Params>
) -> impl IntoResponse {
    info!("{addr} connected!");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, params))
}

async fn handle_socket(mut rawsocket: WebSocket, addr: SocketAddr, state: AppState, params: Params) {
    // save first message
    let mut msg_try = {
        let res = rawsocket.recv().await;
        res
    };

    let rawsocket_wrapped = Arc::new(Mutex::new(Some(rawsocket)));
    let rawsocket_c = rawsocket_wrapped.clone();

    if !SOCKET.set(move || rawsocket_c.clone()) {
        debug!("socket was previously set!");
        let inner = rawsocket_wrapped.lock().await.take();
        SOCKET.get().lock().await.replace(inner.unwrap());
    };

    debug!("Connecting to NATS server for new session by {}", &addr);
    let nats_wrapped = Arc::new(
        Mutex::new(
            Some(
                async_nats::connect(
                    EplOptions::get().nats_addr
                ).await.expect("Failed to connect to the NATS server")
            )
        )
    );
    let nats_c = nats_wrapped.clone();

    if !NATS.set(move || nats_c.clone()) {
        debug!("nats was previously set!");
        let inner = nats_wrapped.lock().await.take();
        NATS.get().lock().await.replace(inner.unwrap());
    }

    // Prepare subscriptions vec
    let nats_subscriptions_wrapped = Arc::new(Mutex::new(Some(vec![])));
    let nats_subscriptions_c = nats_subscriptions_wrapped.clone();

    if !NATS_SUBSCRIPTIONS.set(move || nats_subscriptions_c.clone()) {
        debug!("nats_subscriptions was previously set!");
        let inner = nats_subscriptions_wrapped.lock().await.take();
        NATS_SUBSCRIPTIONS.get().lock().await.replace(inner.unwrap());
    }

    debug!("Connected to NATS server");

    // Do initial unauthed gateway state
    let gateway_state = Arc::new(Mutex::new(Some(GatewayState {
        user_id: None,
        bot: None,
        large_threshold: None,
        current_shard: None,
        shard_count: None,
        intents: None,
        compression: params.compress.map(|compression| compression.parse::<CompressionType>().expect("Invalid compression type requested!")),
        encoding: params.encoding.parse::<EncodingType>().expect("Invalid encoding type requested!"),
    })));
    let gateway_state_c = gateway_state.clone();

    if !GATEWAY_STATE.set(move || gateway_state_c.clone()) {
        let inner = gateway_state.lock().await.take();
        GATEWAY_STATE.get().lock().await.replace(inner.unwrap());
    }

    // Send HELLO to start gateway communication
    send_message(GatewayMessage {
        op: OpCodes::HELLO,
        d: Some(GatewayData::HELLO(Box::from(Hello {
            heartbeat_interval: 10000,
        }))),
        s: None,
        t: None,
    }).await;

    let socket = SOCKET.get();

    let nats = NATS.get();

    let nats_subscriptions = NATS_SUBSCRIPTIONS.get();

    loop {
        // Clippy is being bad here >:(
        // We can't collapse two if lets here as that is still unstable
        #[allow(clippy::collapsible_match)]
        if let Some(msg) = msg_try {
            if let Ok(msg) = msg {
                match msg {
                    Text(msg) => {
                        handle_op(msg, &state).await;
                    }
                    Close(_msg) => {
                        info!("bye bye {addr}");
                        break;
                    }
                    Ping(_msg) => {
                        debug!("Ping from {addr}")
                    }
                    Pong(_msg) => {
                        debug!("Pong from {addr}")
                    }
                    _ => {
                        debug!("Bad gateway message from {addr}!");
                        break;
                    }
                }
            } else {
                info!("bye bye {addr} (closed due to error {msg:#?})");
                break;
            }
        }

        // Ensure all NATS messages are sent and then build queue of subscriptions to process
        let mut nats_lock = nats.lock().await;
        let nats_operable = nats_lock.as_mut().unwrap();

        nats_operable.flush().await.expect("Failed to flush NATS message queue!");

        let mut nats_subscriptions_lock = nats_subscriptions.lock().await;
        let nats_subscriptions_operable = nats_subscriptions_lock.as_mut().unwrap();

        let mut nats_messages: Vec<async_nats::Message> = vec![];

        for i in nats_subscriptions_operable {
            if let Some(message) = i.next().await {
                debug!("Received NATS message: {:?}", &message);

                nats_messages.push(message);
            }
        }

        drop(nats_subscriptions_lock);
        drop(nats_lock);

        for i in nats_messages {
            todo!()
        }

        // Capture next websocket message
        msg_try = {
            let mut socket_lock = socket.lock().await;
            let res = socket_lock.as_mut().unwrap().recv().await;
            drop(socket_lock);
            res
        };
    }
}
