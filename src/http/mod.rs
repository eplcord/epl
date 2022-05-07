use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use log::info;
use rocket::config::Ident;
use rocket_dyn_templates::Template;
use rocket::figment::{value::{Map, Value}, util::map};

use crate::{EplOptions, Options, VERSION};
use rocket::options;

mod routes;
mod cors;

#[options("/<_path..>")]
pub async fn cors_options(_path: PathBuf) -> &'static str {
    ""
}

pub async fn entry() {
    info!("Hello from the HTTP API!");

    let options = EplOptions::get();

    let listen_addr = options.http_listen_addr.split_once(":")
        .expect("Issue getting HTTP Listen Address!");

    let db: Map<_, Value> = map!{
        "url" => options.database_url.into(),
        "pool_size" => 12.into()
    };

    let rocket_options = rocket::Config::figment()
        // Database
        .merge(("databases", map!["epl_db" => db]))

        // Server configuration
        .merge(("address", IpAddr::from_str(listen_addr.0)
            .expect("IP has incorrect format!")))
        .merge(("port", u16::from_str(listen_addr.1)
            .expect("Port has incorrect format!")))

        // Branding
        .merge(("ident", Ident::try_new(format!("Epl v{}", VERSION))
            .expect("Failed to create new Ident")));

    rocket::custom(rocket_options)
        // CORS routes
        .mount("/", rocket::routes![
            cors_options
        ])
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
        .attach(crate::database::EplDb::fairing())
        .attach(cors::CORS)

        .launch()
        .await
        .expect("Failed to start the HTTP API server!");
}