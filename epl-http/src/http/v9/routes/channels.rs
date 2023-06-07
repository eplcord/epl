use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use epl_common::Stub;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GetMessageRes(Vec<Stub>);

pub async fn get_messages(
    Extension(_state): Extension<AppState>,
    Extension(_session_context): Extension<SessionContext>,
    Path(_channel_id): Path<i64>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(GetMessageRes(vec![])))
}

#[derive(Deserialize)]
pub struct SendMessageReq {
    content: String,
    flags: i64,
    nonce: String,
    tts: bool,
}

pub async fn send_message(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
    Json(message): Json<SendMessageReq>,
) -> impl IntoResponse {

}