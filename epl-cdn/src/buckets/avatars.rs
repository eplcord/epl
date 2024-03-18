use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
use axum::body::{Body, StreamBody};
use axum::Extension;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use tracing::log::debug;
use crate::AppState;

use tokio_util::io::ReaderStream;
use tracing::error;

pub async fn avatars(
    Path((user_id, file)): Path<(u64, String)>,
    Extension(state): Extension<AppState>
) -> impl IntoResponse {
    // TODO: Resize image to what the client wants
    // TODO: Probably do this on upload to specific res (16, 32, 80) and then on the fly with cache for others
    debug!("Hello! You wanted {user_id}'s avatar with the filename {file}!");

    let object = state.aws
        .get_object()
        .bucket("avatars")
        .key(format!("{user_id}/{file}"))
        .send()
        .await;

    match object {
        Ok(object) => {
            let stream = ReaderStream::new(object.body.into_async_read());

            let body = StreamBody::new(stream);

            let headers = [
                // TODO: detect this and fix this :)
                (header::CONTENT_TYPE, "image/webp"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{file}\"")),
            ];

            (headers, body).into_response()
        }
        Err(error) => {
            error!("{error}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}