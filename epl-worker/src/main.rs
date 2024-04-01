mod handle;

use std::env;
use futures::StreamExt;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tracing::{debug, info, error, log};
use epl_common::options::{EplOptions, Options};
use epl_common::rustflake;
use migration::Migrator;
use crate::handle::handle_nats_message;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// const URL_REGEX: &str = "https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = EplOptions::get();
    let mut snowflake_factory = rustflake::Snowflake::default();

    info!("Starting epl-gateway v{}", VERSION);
    debug!("Starting on {}", snowflake_factory.generate());

    info!("\tName: {}", options.name);
    info!("\tURL: {}", options.url);
    info!("\tGateway URL: {}", options.gateway_url);
    if options.mediaproxy_url.is_some() {
        info!("\tMediaproxy URL: {}", options.mediaproxy_url.clone().unwrap());
    }
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

    // Workaround for https://github.com/awslabs/aws-sdk-rust/issues/932
    let aws_config = if env::var("AWS_ENDPOINT_URL").is_ok() {
        aws_config::from_env().endpoint_url(env::var("AWS_ENDPOINT_URL").unwrap()).load().await
    } else {
        aws_config::load_from_env().await
    };

    let aws = aws_sdk_s3::Client::new(&aws_config);

    info!("Loaded S3 configuration");

    let stream = client.subscribe("worker_queue").await;
    if let Err(_) = stream {
        // handle error
        return;
    }
    let mut stream = stream.unwrap();

    while let Some(message) = stream.next().await {
        let appstate = AppState {
            db: conn.clone(),
            nats: client.clone(),
            aws: aws.clone(),
            options: options.clone()
        };

        tokio::spawn(async move{
            if let Ok(message) = serde_json::from_slice::<epl_common::nats::Messages>(&message.payload) {
                handle_nats_message(&appstate, message).await;
            } else {
                error!("Unable to deserialize received message!");
            }
        });
    }
}

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    nats: async_nats::Client,
    aws: aws_sdk_s3::Client,
    options: EplOptions
}
