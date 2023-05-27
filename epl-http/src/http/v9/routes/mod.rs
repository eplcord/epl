mod tracking;
mod auth;
mod users;

use axum::{middleware, Router};
use axum::routing::{get, post};
use crate::authorization_extractor::get_session_context;
use crate::http::v9::routes::auth::{location_metadata, login, logout, register, verify_email};
use crate::http::v9::routes::users::profile;

pub fn assemble_routes() -> Router {
    let authenticated_auth = Router::new()
        .route("/logout", post(logout))

        .route("/verify", post(verify_email))
        .route("/verify/resend", post(verify_email))

        .route_layer(middleware::from_fn(get_session_context));

    let auth = Router::new()
        .route("/location-metadata", get(location_metadata))

        .route("/login", post(login))
        .route("/register", post(register))

        .merge(authenticated_auth);

    let users = Router::new()
        .route("/:id/profile", get(profile))

        .route_layer(middleware::from_fn(get_session_context));

    Router::new()
        .nest("/auth", auth)
        .nest("/users", users)

        .route("/experiments", get(tracking::experiments))

        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}