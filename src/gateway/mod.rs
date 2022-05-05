use std::net::SocketAddr;
use std::str::FromStr;

use log::info;
use rocket::futures::StreamExt;
use warp::Filter;
use warp::ws::WebSocket;

use crate::{EplOptions, Options};
use crate::gateway::handle_op::handle_op;

mod schema;
mod handle_op;

async fn accept(ws: WebSocket) {
    let (mut write, mut read) = ws.split();
    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg.expect("Bad gateway message!");
                        if msg.is_text() {
                            handle_op(msg, &mut write).await;
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
}

pub async fn entry() {
    info!("Hello from the Gateway!");

    let options = EplOptions::get();

    let socket = warp::path::end()
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(move |socket| accept(socket))
        });
    info!("Gateway is active on {}!", &options.gateway_listen_addr);

    warp::serve(socket).run(SocketAddr::from_str(&options.gateway_listen_addr)
        .expect("Failed to start Gateway!"))
        .await;
}