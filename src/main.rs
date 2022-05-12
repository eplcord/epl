#[macro_use] extern crate diesel;
extern crate core;

use diesel::pg::PgConnection;
use log::{debug, info};
use pretty_env_logger;
use tokio::join;

use options::{EplOptions, Options};
use crate::util::rustflake;

mod options;
mod gateway;
mod http;
mod database;
mod util;
mod schema;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[rocket::main]
async fn main() {
    pretty_env_logger::init();

    let options = EplOptions::get();
    let mut snowflake_factory = rustflake::Snowflake::default();

    info!("Starting epl v{}", VERSION);
    debug!("Starting on {}", snowflake_factory.generate());

    info!("\tName: {}", options.name);
    info!("\tURL: {}", options.url);
    info!("\tGateway URL: {}", options.gateway_url);
    if options.mediaproxy_url.is_some() {
        info!("\tMediaproxy URL: {}", options.mediaproxy_url.unwrap());
    }
    info!("\tHTTP Listen Address: {}", options.http_listen_addr);
    info!("\tGateway Listen Address: {}", options.gateway_listen_addr);
    info!("\tRequire SSL: {}", options.require_ssl);

    // Even though Rocket will handle its own database pool, we will still need one for the Gateway
    // We might consider just using this pool and plugging it into Rocket's state in the future
    info!("Connecting to database");
    let db_manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(options.database_url);
    let db_pool = diesel::r2d2::Pool::builder()
        .max_size(12) // Keep this in sync with Rocket (Move it to env?)
        .build(db_manager)
        .expect("Failed to connect to the database!");

    info!("Spawning HTTP API");
    let http = tokio::spawn(async {
        http::entry().await
    });

    info!("Spawning Gateway");
    let gateway = tokio::spawn( async move {
        gateway::entry(db_pool).await
    });

    let res = join!(http, gateway);
    res.0.expect("Failed to join the HTTP API server!");
    res.1.expect("Failed to join the Gateway server!")
}
