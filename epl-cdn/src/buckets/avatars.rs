use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use serde_derive::Deserialize;
use tracing::log::debug;

use crate::AppState;
use crate::buckets::query_cached_size_or_create;

#[derive(Deserialize)]
pub struct AvatarsQuery {
    pub size: Option<u32>
}


pub async fn avatars(
    Path((user_id, file)): Path<(u64, String)>,
    Extension(state): Extension<AppState>,
    path_query: Query<AvatarsQuery>
) -> impl IntoResponse {
    debug!("Hello! You wanted {user_id}'s avatar with the filename {file}!");

    query_cached_size_or_create("avatars", &state, user_id, file, path_query.size).await
}