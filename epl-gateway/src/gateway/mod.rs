use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

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
use crate::state::{GatewayState, EncodingType, CompressionType, ThreadData};

use axum_client_ip::SecureClientIp;

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
    SecureClientIp(addr): SecureClientIp,
    Extension(state): Extension<AppState>,
    Query(params): Query<Params>
) -> impl IntoResponse {
    info!("{addr} connected!");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, params))
}

async fn handle_socket(mut rawsocket: WebSocket, addr: IpAddr, state: AppState, params: Params) {
    // save first message
    let mut msg_try = {
        let res = rawsocket.recv().await;
        res
    };

    let mut thread_data = ThreadData {
        session_ip: Some(addr),
        socket: Some(rawsocket),
        ..Default::default()
    };

    debug!("Connecting to NATS server for new session by {}", &addr);
    let nats_wrapped =
            Some(
                async_nats::connect(
                    EplOptions::get().nats_addr
                ).await.expect("Failed to connect to the NATS server")
    );

    thread_data.nats = nats_wrapped;

    // Prepare subscriptions vec
    let nats_subscriptions_wrapped = Some(vec![]);

    thread_data.nats_subscriptions = nats_subscriptions_wrapped;

    debug!("Connected to NATS server");

    // Do initial unauthed gateway state
    let gateway_state = Some(GatewayState {
        user_id: None,
        bot: None,
        large_threshold: None,
        current_shard: None,
        shard_count: None,
        intents: None,
        compression: params.compress.map(|compression| compression.parse::<CompressionType>().expect("Invalid compression type requested!")),
        encoding: params.encoding.parse::<EncodingType>().expect("Invalid encoding type requested!"),
    });

    thread_data.gateway_state = gateway_state;

    // Send HELLO to start gateway communication
    send_message(&mut thread_data, GatewayMessage {
        op: OpCodes::HELLO,
        d: Some(GatewayData::HELLO(Box::from(Hello {
            heartbeat_interval: 10000,
        }))),
        s: None,
        t: None,
    }).await;

    loop {
        // Clippy is being bad here >:(
        // We can't collapse two if lets here as that is still unstable
        #[allow(clippy::collapsible_match)]
        if let Some(msg) = msg_try {
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

        // Ensure all NATS messages are sent and then build queue of subscriptions to process
        let nats_operable = thread_data.nats.as_mut().unwrap();

        nats_operable.flush().await.expect("Failed to flush NATS message queue!");

        let nats_subscriptions_operable = thread_data.nats_subscriptions.as_mut().unwrap();

        let mut nats_messages: Vec<async_nats::Message> = vec![];

        for i in nats_subscriptions_operable {
            if let Some(message) = i.next().await {
                debug!("Received NATS message: {:?}", &message);

                nats_messages.push(message);
            }
        }

        for i in nats_messages {
            todo!()
        }

        // Capture next websocket message
        msg_try = {
            thread_data.socket.as_mut().unwrap().recv().await
        };
    }
}
