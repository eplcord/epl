mod avatars;

use axum::Router;
use axum::routing::get;
use crate::buckets::avatars::avatars;

pub fn buckets() -> Router {
    Router::new()
        // User Avatars
        .route("/avatars/:user_id/:file", get(avatars))
}