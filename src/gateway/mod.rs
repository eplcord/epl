use log::info;

use std::future;
use rocket::futures::{StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use crate::{EplOptions, Options};

// TODO: Remove test/example code
async fn accept(stream: TcpStream) {
    let peer_addr = stream.peer_addr().expect("Peer doesn't have IP address!");
    info!("Incoming connection from {}", &peer_addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error with socket handshake!");

    info!("Connected with {}!", &peer_addr);

    let (write, read) = ws_stream.split();

    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward message!")
}

pub async fn entry() {
    info!("Hello from the Gateway!");

    let options = EplOptions::get();

    let listen = TcpListener::bind(&options.gateway_listen_addr)
        .await
        .expect("Failed to bind to Gateway Listen Address!");

    info!("Gateway is active on {}!", &options.gateway_listen_addr);

    while let Ok((stream, _)) = listen.accept().await {
        tokio::spawn(accept(stream));
    }
}