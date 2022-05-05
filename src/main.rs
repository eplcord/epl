use pretty_env_logger;
use log::info;

use tokio::join;

use options::{EplOptions, Options};

mod options;
mod gateway;
mod http;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let options = EplOptions::get();

    info!("Starting epl v{}", VERSION);

    info!("\tName: {}", options.name);
    info!("\tURL: {}", options.url);
    info!("\tGateway URL: {}", options.gateway_url);
    if options.mediaproxy_url.is_some() {
        info!("\tMediaproxy URL: {}", options.mediaproxy_url.unwrap());
    }
    info!("\tHTTP Listen Address: {}", options.http_listen_addr);
    info!("\tGateway Listen Address: {}", options.gateway_listen_addr);
    info!("\tRequire SSL: {}", options.require_ssl);

    info!("Spawning HTTP API");
    let http = tokio::spawn(async {
        http::entry().await
    });

    info!("Spawning Gateway");
    let gateway = tokio::spawn(async {
        gateway::entry().await
    });

    let res = join!(http, gateway);
    res.0.expect("Failed to join the HTTP API server!");
    res.1.expect("Failed to join the Gateway server!")
}
