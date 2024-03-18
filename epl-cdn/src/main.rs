mod buckets;

use std::env;
use std::net::SocketAddr;
use aws_sdk_s3::Client;
use axum::http::Method;
use axum::{Extension, Router};
use axum::routing::get;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, log};
use epl_common::options::{EplOptions, Options};
use epl_common::rustflake;
use migration::Migrator;
use crate::buckets::buckets;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = EplOptions::get();
    let mut snowflake_factory = rustflake::Snowflake::default();

    info!("Starting epl-cdn v{}", VERSION);
    debug!("Starting on {}", snowflake_factory.generate());

    info!("\tName: {}", options.name);
    info!("\tURL: {}", options.url);
    info!("\tListen Address: {}", options.listen_addr);
    info!("\tRequire SSL: {}", options.require_ssl);

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
    
    info!("Loading S3 configuration");
    
    // Workaround for https://github.com/awslabs/aws-sdk-rust/issues/932
    let aws_config = if env::var("AWS_ENDPOINT_URL").is_ok() {
        aws_config::from_env().endpoint_url(env::var("AWS_ENDPOINT_URL").unwrap()).load().await
    } else {
        aws_config::load_from_env().await
    };
    
    let aws = aws_sdk_s3::Client::new(&aws_config);

    info!("Loaded S3 configuration");

    info!("Starting server");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::OPTIONS,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_headers(Any);

    let app_state = AppState {
        conn,
        aws
    };

    let app = Router::new()
        .nest("/", buckets())
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
    aws: Client
}

async fn index() -> &'static str {
    "hello from the epl cdn :3"
}