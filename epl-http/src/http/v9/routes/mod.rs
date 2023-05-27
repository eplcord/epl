mod tracking;
mod auth;

use axum::{middleware, Router};
use axum::routing::{get, post};
use crate::authorization_extractor::get_session_context;
use crate::http::v9::routes::auth::{location_metadata, login, logout, register};

pub fn assemble_routes() -> Router {
    let authenticated_auth = Router::new()
        .route("/logout", post(logout))

        .route_layer(middleware::from_fn(get_session_context));

    let auth = Router::new()
        .route("/location-metadata", get(location_metadata))

        .route("/login", post(login))
        .route("/register", post(register))

        .merge(authenticated_auth);

    Router::new()
        .nest("/auth", auth)

        .route("/experiments", get(tracking::experiments))

        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}