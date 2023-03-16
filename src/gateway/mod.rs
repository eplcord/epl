use std::net::SocketAddr;

use axum::{Extension, extract::ws::{Message, WebSocket, WebSocketUpgrade}, extract::ws::Message::{Text, Close, Ping, Pong}, response::IntoResponse};
use axum::extract::connect_info::ConnectInfo;

use futures::stream::StreamExt;

use tracing::{debug, info};
use crate::AppState;

use crate::gateway::handle::handle_op;

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

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, state: AppState) {
    // Check connection with socket
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Connection with {addr} is ok.")
    } else {
        debug!("Could not ping {addr}, dropping.");
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg.expect("Bad gateway message from {addr}!");
                        match msg {
                            Text(msg) => {
                                handle_op(msg, &mut sender, &state).await;
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
                    None => break,
                }
            }
        }
    }
}
