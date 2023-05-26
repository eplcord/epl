mod tracking;

use axum::Router;
use axum::routing::post;

pub fn assemble_routes() -> Router {
    let auth = Router::new();

    Router::new()
        .nest("/auth", auth)

        .route("/track", post(tracking::track))
}