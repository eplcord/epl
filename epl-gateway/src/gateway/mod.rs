use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::ws::Message::{Close, Ping, Pong, Text},
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};

use crate::AppState;
use tracing::{debug, info};

use crate::gateway::handle::handle_op;
use crate::gateway::schema::hello::Hello;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};
use crate::gateway::schema::GatewayMessage;
use crate::state::{WebSocketWrapper, SOCKET};

mod dispatch;
mod handle;
mod schema;

pub async fn gateway(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    info!("{addr} connected!");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut rawsocket: WebSocket, addr: SocketAddr, state: AppState) {
    // save first message
    let mut msg_try = {
        let res = rawsocket.recv().await;
        res
    };
    
    // Send HELLO and check connection with socket
    if rawsocket
        .send(Message::Text(
            serde_json::to_string(&GatewayMessage {
                op: OpCodes::HELLO,
                d: Some(GatewayData::HELLO(Box::from(Hello {
                    heartbeat_interval: 10000,
                }))),
                s: None,
                t: None,
            })
            .expect("Failed to serialize HELLO!"),
        ))
        .await
        .is_ok()
    {
        debug!("Connection with {addr} is ok.");
    } else {
        debug!("Could not ping {addr}, dropping.");
        return;
    }

    let rawsocket_wrapped = Arc::new(Mutex::new(Some(WebSocketWrapper { inner: rawsocket })));
    let rawsocket_c = rawsocket_wrapped.clone();

    if !SOCKET.set(move || rawsocket_c.clone()) {
        debug!("socket was previously set!");
        let inner = rawsocket_wrapped.lock().await.take();
        SOCKET.get().lock().await.replace(inner.unwrap());
    };

    let socket = SOCKET.get();

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
        
        msg_try = {
            let mut socket_lock = socket.lock().await;
            let res = socket_lock.as_mut().unwrap().inner.recv().await;
            drop(socket_lock);
            res
        };
    }
}
