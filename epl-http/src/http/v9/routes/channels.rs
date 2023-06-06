use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_derive::Serialize;
use epl_common::Stub;
use crate::AppState;
use crate::authorization_extractor::SessionContext;

#[derive(Serialize)]
pub struct GetMessageRes(Vec<Stub>);

pub async fn get_messages(
    Extension(_state): Extension<AppState>,
    Extension(_session_context): Extension<SessionContext>,
    Path(_channel_id): Path<i64>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(GetMessageRes(vec![])))
}