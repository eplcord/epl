mod tracking;
mod auth;

use axum::Router;
use axum::routing::{get, post};
use crate::http::v9::routes::auth::{location_metadata, login, register};

pub fn assemble_routes() -> Router {
    let auth = Router::new()
        .route("/location-metadata", get(location_metadata))

        .route("/login", post(login))
        .route("/register", post(register));

    Router::new()
        .nest("/auth", auth)

        .route("/experiments", get(tracking::experiments))

        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}