mod tracking;
mod auth;
mod users;
mod hypesquad;

use axum::{middleware, Router};
use axum::routing::{delete, get, post};
use crate::authorization_extractor::get_session_context;
use crate::http::v9::routes::auth::{location_metadata, login, logout, register, sessions, verify_email};
use crate::http::v9::routes::hypesquad::{join_hypesquad, leave_hypesquad};
use crate::http::v9::routes::users::profile;

pub fn assemble_routes() -> Router {
    let authenticated_auth = Router::new()
        .route("/logout", post(logout))

        .route("/verify", post(verify_email))
        .route("/verify/resend", post(verify_email))

        .route("/sessions", get(sessions))

        .route_layer(middleware::from_fn(get_session_context));

    let auth = Router::new()
        .route("/location-metadata", get(location_metadata))

        .route("/login", post(login))
        .route("/register", post(register))

        .merge(authenticated_auth);

    let users = Router::new()
        .route("/:id/profile", get(profile))

        .route_layer(middleware::from_fn(get_session_context));

    let hypesquad = Router::new()
        .route("/online", post(join_hypesquad))
        .route("/online", delete(leave_hypesquad))

        .route_layer(middleware::from_fn(get_session_context));

    Router::new()
        .nest("/auth", auth)
        .nest("/users", users)

        .nest("/hypesquad", hypesquad)

        .route("/experiments", get(tracking::experiments))

        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}