mod routes;

use std::net::IpAddr;
use std::str::FromStr;
use log::info;
use rocket_dyn_templates::Template;

use crate::{EplOptions, Options};

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
        // Index routes
        .mount("/", rocket::routes![
            routes::index::index
        ])

        // API v9 routes
        .mount("/api/v9", rocket::routes![
            routes::v9::experiments::experiments,
            routes::v9::experiments::science
        ])

        // Legacy API v6 routes
        .mount("/api/v6", rocket::routes![
            routes::v6::experiments::experiments,
            routes::v6::experiments::science,
            routes::v6::experiments::track
        ])

        // Fairings
        .attach(Template::fairing())

        .launch()
        .await
        .expect("Failed to start the HTTP API server!");
}