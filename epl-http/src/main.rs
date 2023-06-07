extern crate core;

use askama::Template;
use async_nats::Client;
use axum::http::Method;
use axum::routing::get;
use axum::{Extension, Router};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, log};

use crate::http::api;
use epl_common::options::{EplOptions, Options};
use epl_common::rustflake;

use migration::{Migrator, MigratorTrait};

mod authorization_extractor;
mod http;
mod nats;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = EplOptions::get();
    let mut snowflake_factory = rustflake::Snowflake::default();

    info!("Starting epl-http v{}", VERSION);
    debug!("Starting on {}", snowflake_factory.generate());

    info!("\tName: {}", options.name);
    info!("\tURL: {}", options.url);
    info!("\tGateway URL: {}", options.gateway_url);
    if options.mediaproxy_url.is_some() {
        info!("\tMediaproxy URL: {}", options.mediaproxy_url.unwrap());
    }
    info!("\tListen Address: {}", options.listen_addr);
    info!("\tRequire SSL: {}", options.require_ssl);

    info!("\tNATS Address: {}", options.nats_addr);

    info!("Connecting to database");

    let mut migration_db_opt =
        migration::sea_orm::ConnectOptions::new(options.database_url.clone());
    migration_db_opt.sqlx_logging_level(log::LevelFilter::Debug);

    info!("Checking for migrations needed");
    let migrator_conn = migration::sea_orm::Database::connect(migration_db_opt)
        .await
        .expect("Failed to connect to the database!");
    Migrator::up(&migrator_conn, None)
        .await
        .expect("Failed to run migrations!");

    let mut db_opt = ConnectOptions::new(options.database_url.clone());
    db_opt.sqlx_logging_level(log::LevelFilter::Debug);

    let conn: DatabaseConnection = Database::connect(db_opt)
        .await
        .expect("Failed to connect to database!");

    info!("Connected to database");

    info!("Connecting to NATS server");

    let client = async_nats::connect(EplOptions::get().nats_addr)
        .await
        .expect("Failed to connect to NATS server");

    info!("Connected to NATS server");

    info!("Starting server");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::OPTIONS,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_headers(Any);

    let app_state = AppState {
        conn,
        nats_client: client,
    };

    let app = Router::new()
        .nest("/api", api())
        .route("/", get(index))
        .layer(cors)
        .layer(Extension(app_state));

    let addr: SocketAddr = options
        .listen_addr
        .parse()
        .expect("Unable to parse listen address!");

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to start the server!");
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
    nats_client: Client,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    instance_name: String,
    version: String,
}

async fn index() -> IndexTemplate {
    let options = EplOptions::get();

    IndexTemplate {
        instance_name: options.name,
        version: VERSION.to_string(),
    }
}
