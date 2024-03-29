use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use axum::{response::IntoResponse, Extension};

use axum_tungstenite::Message::{Close, Ping, Pong, Text};
use axum_tungstenite::{WebSocket, WebSocketUpgrade};

use axum::extract::Query;
use futures::{FutureExt, StreamExt};
use serde::{de, Deserialize, Deserializer};

use crate::gateway::dispatch::send_message;
use crate::AppState;
use epl_common::options::{EplOptions, Options};
use tracing::{debug, info};

use crate::gateway::handle::handle_op;
use crate::gateway::schema::hello::Hello;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};
use crate::gateway::schema::GatewayMessage;
use crate::state::{CompressionType, EncodingType, GatewayState, ThreadData};

use crate::gateway::nats::handle_nats_message;
use axum_client_ip::SecureClientIp;
use epl_common::rustflake::Snowflake;

mod dispatch;
mod handle;
mod nats;
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
    pub compress: Option<String>,
}

pub async fn gateway(
    ws: WebSocketUpgrade,
    SecureClientIp(addr): SecureClientIp,
    Extension(state): Extension<AppState>,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    info!("{addr} connected!");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, params))
}

async fn handle_socket(mut rawsocket: WebSocket, addr: IpAddr, state: AppState, params: Params) {
    // save first message
    let mut msg_try = { rawsocket.recv().now_or_never() };

    debug!("Connecting to NATS server for new session by {}", &addr);
    let nats = async_nats::connect(EplOptions::get().nats_addr)
        .await
        .expect("Failed to connect to the NATS server");

    // Prepare subscriptions vec
    let nats_subscriptions = HashMap::new();

    debug!("Connected to NATS server");

    // Do initial unauthed gateway state
    let gateway_state = GatewayState {
        gateway_session_id: Snowflake::default().generate(),
        user_id: None,
        session_id: None,
        bot: None,
        large_threshold: None,
        current_shard: None,
        shard_count: None,
        intents: None,
        compression: params.compress.map(|compression| {
            compression
                .parse::<CompressionType>()
                .expect("Invalid compression type requested!")
        }),
        encoding: params
            .encoding
            .parse::<EncodingType>()
            .expect("Invalid encoding type requested!"),
        sequence: 1,
    };

    let mut thread_data = ThreadData {
        gateway_state,
        session_ip: addr,
        socket: rawsocket,
        nats,
        snowflake_factory: Snowflake {
            ..Default::default()
        },
        nats_subscriptions,
    };

    // Send HELLO to start gateway communication
    send_message(
        &mut thread_data,
        GatewayMessage {
            op: OpCodes::Hello,
            d: Some(GatewayData::Hello(Box::from(Hello {
                heartbeat_interval: 10000,
            }))),
            s: Some(0),
            t: None,
        },
    )
    .await;

    loop {
        // Clippy is being bad here >:(
        // We can't collapse two if lets here as that is still unstable
        #[allow(clippy::collapsible_match)]
        if let Some(msg) = msg_try {
            if let Some(msg) = msg {
                if let Ok(msg) = msg {
                    match msg {
                        Text(msg) => {
                            handle_op(&mut thread_data, msg, &state).await;
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
        }

        // Ensure all NATS messages are sent
        thread_data
            .nats
            .flush()
            .await
            .expect("Failed to flush NATS message queue!");

        let mut nats_messages: Vec<async_nats::Message> = vec![];

        for i in thread_data.nats_subscriptions.iter_mut() {
            // Clippy is being bad here >:( again :(
            // We can't collapse two if lets here as that is still unstable
            #[allow(clippy::collapsible_match)]
            if let Some(message) = i.1.next().now_or_never() {
                if let Some(message) = message {
                    debug!("Received NATS message: {:?}", &message);

                    nats_messages.push(message);
                }
            }
        }

        for i in nats_messages {
            if let Ok(msg) = serde_json::from_slice::<epl_common::nats::Messages>(&i.payload) {
                handle_nats_message(&mut thread_data, msg, &state).await;
            } else {
                debug!("huhhh?");
            }
        }

        // Capture next websocket message
        msg_try = thread_data.socket.recv().now_or_never()
    }
}
