use axum::body::StreamBody;
use axum::Extension;
use axum::extract::{Path};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use tokio_util::io::ReaderStream;
use tracing::log::debug;
use epl_common::options::{EplOptions, Options};
use crate::AppState;

pub async fn badge_icons(
    Path(file): Path<String>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    debug!("Hello! You wanted the {file} badge icon!");

    let object = state.aws
        .get_object()
        .bucket(EplOptions::get().s3_bucket)
        .key(format!("badge-icons/{file}"))
        .send()
        .await;

    match object {
        Ok(object) => {
            let stream = ReaderStream::new(object.body.into_async_read());

            let body = StreamBody::new(stream);

            let headers = [
                // TODO: detect this and fix this :)
                (header::CONTENT_TYPE, "image/png"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{file}\"")),
            ];

            (headers, body).into_response()
        }
        Err(e) => {
            StatusCode::NOT_FOUND.into_response()
        }
    }
}