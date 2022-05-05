use std::net::IpAddr;
use std::str::FromStr;
use log::info;

use rocket::{get, routes};
use crate::{EplOptions, Options};

// TODO: Remove test/example code
#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

pub async fn entry() {
    info!("Hello from the HTTP API!");

    let options = EplOptions::get();

    let listen_addr = options.http_listen_addr.split_once(":")
        .expect("Issue getting HTTP Listen Address!");

    let rocket_options = rocket::Config {
        address: IpAddr::from_str(listen_addr.0)
            .expect("IP has incorrect format!"),
        port: u16::from_str(listen_addr.1)
            .expect("Port has incorrect format!"),
        ..rocket::Config::release_default()
    };

    rocket::custom(&rocket_options)
        .mount("/", routes![index])
        .launch()
        .await
        .expect("Failed to start the HTTP API server!");
}