use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

use async_bb8_diesel::ConnectionManager;
use bb8::Pool;
use diesel::PgConnection;
use log::info;
use rocket::futures::StreamExt;
use warp::Filter;
use warp::ws::WebSocket;

use crate::{EplOptions, Options};
use crate::gateway::handle_op::handle_op;

mod schema;
mod handle_op;

type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

fn with_pool(
    pool: ConnectionPool,
) -> impl Filter<Extract = (ConnectionPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn accept(ws: WebSocket, pool: ConnectionPool) {
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

pub async fn entry(db_pool: Pool<ConnectionManager<PgConnection>>) {
    info!("Hello from the Gateway!");

    let options = EplOptions::get();

    let socket = warp::path::end()
        .and(warp::ws())
        .and(with_pool(db_pool.clone()))
        .map(|ws: warp::ws::Ws, db_pool: Pool<ConnectionManager<PgConnection>> | {
            ws.on_upgrade(move |socket | accept(socket, db_pool))
        });
    info!("Gateway is active on {}!", &options.gateway_listen_addr);

    warp::serve(socket).run(SocketAddr::from_str(&options.gateway_listen_addr)
        .expect("Failed to start Gateway!"))
        .await;
}