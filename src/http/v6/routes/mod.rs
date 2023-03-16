mod tracking;

use axum::Router;
use axum::routing::{get, post};

pub fn assemble_routes() -> Router {
    let auth = Router::new();

    Router::new()
        .nest("/auth", auth)

        .route("/experiments", get(tracking::experiments))

        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}