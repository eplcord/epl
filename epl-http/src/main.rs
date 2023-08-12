extern crate core;

use askama::Template;
use async_nats::Client;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use epl_common::database::entities::prelude::{Channel, Message, User};
use epl_common::nodeinfo::{LitecordMetadata, NodeInfo, Services, Software, Usage, UsageUsers};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait, PaginatorTrait};
use serde_derive::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, log};

use crate::http::api;
use epl_common::options::{EplOptions, Options};
use epl_common::{rustflake, Stub};

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
        .route("/nodeinfo/2.1.json", get(nodeinfo))
        .route("/.well-known/nodeinfo", get(well_known_nodeinfo))
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
    instance_description: String,
    version: String,
    message_count: u64,
    channel_count: u64,
    user_count: u64,
    guild_count: u64,
}

async fn index(Extension(state): Extension<AppState>) -> IndexTemplate {
    let options = EplOptions::get();

    let message_count = Message::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    let channel_count = Channel::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    let user_count = User::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    IndexTemplate {
        instance_name: options.name,
        instance_description: options.description,
        version: VERSION.to_string(),
        message_count,
        channel_count,
        user_count,
        guild_count: 0,
    }
}

#[derive(Serialize)]
struct WellKnownNodeInfoRes {
    links: Vec<WellKnownLink>,
}

#[derive(Serialize)]
struct WellKnownLink {
    rel: String,
    href: String,
}

async fn well_known_nodeinfo() -> impl IntoResponse {
    let options = EplOptions::get();

    Json(WellKnownNodeInfoRes {
        links: vec![WellKnownLink {
            rel: "http://nodeinfo.diaspora.software/ns/schema/2.1".to_string(),
            href: format!(
                "{}://{}/nodeinfo/2.1.json",
                if options.require_ssl { "https" } else { "http" },
                options.url
            ),
        }],
    })
}

async fn nodeinfo(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let options = EplOptions::get();

    let user_count = User::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    let message_count = Message::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    Json(NodeInfo {
        version: "2.1".to_string(),
        software: Software {
            name: "Epl".to_string(),
            version: VERSION.to_string(),
            repository: Some("https://git.gaycatgirl.sex/epl/epl".to_string()),
            homepage: None,
        },
        protocols: vec![],
        services: Services {
            inbound: vec![],
            outbound: vec![],
        },
        open_registrations: options.registration,
        usage: Usage {
            users: UsageUsers {
                total: user_count,
                active_half_year: None,
                active_month: None,
            },
            local_posts: Some(message_count),
            local_comments: None,
        },
        metadata: LitecordMetadata {
            node_name: options.name,
            node_description: options.description,
            private: false,
            features: vec!["discord_api".to_string()],
            federation: Stub {},
        },
    })
}
