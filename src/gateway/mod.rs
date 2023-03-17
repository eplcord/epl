use std::net::SocketAddr;
use std::sync::{Arc};
use tokio::sync::Mutex;

use axum::{Extension, extract::ws::{Message, WebSocket, WebSocketUpgrade}, extract::ws::Message::{Text, Close, Ping, Pong}, response::IntoResponse};
use axum::extract::connect_info::ConnectInfo;

use futures::TryStreamExt;

use tracing::{debug, info};
use crate::AppState;

use crate::gateway::handle::handle_op;
use crate::state::{SOCKET, WebSocketWrapper};

mod schema;
mod handle;
mod dispatch;

pub async fn gateway(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<AppState>
) -> impl IntoResponse {
    info!("{addr} connected!");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut rawsocket: WebSocket, addr: SocketAddr, state: AppState) {
    // Check connection with socket
    if rawsocket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Connection with {addr} is ok.")
    } else {
        debug!("Could not ping {addr}, dropping.");
        return;
    }

    let rawsocket_wrapped = Arc::new(Mutex::new(WebSocketWrapper { inner: rawsocket }));

    SOCKET.set(move || rawsocket_wrapped.clone());

    let socket = SOCKET.get();

    loop {
        let msg_try = {
            let mut socket_lock = socket.lock().await;
            socket_lock.inner.try_next().await
        };

        // Clippy is being bad here >:(
        // We can't collapse two if lets here as that is still unstable
        #[allow(clippy::collapsible_match)]
        if let Ok(msg) = msg_try {
            if let Some(msg) = msg {
                match msg {
                    Text(msg) => {
                        handle_op(msg, &state).await;
                    },
                    Close(_msg) => {
                        info!("bye bye {addr}");
                        break;
                    },
                    Ping(_msg) => {
                        debug!("Ping from {addr}")
                    },
                    Pong(_msg) => {
                        debug!("Pong from {addr}")
                    }
                    _ => {
                        debug!("Bad gateway message from {addr}!");
                        break;
                    }
                }
            }
        }
    }
}
