use axum::body::Body;
use axum::Extension;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use sea_orm::EntityTrait;
use tokio_util::io::ReaderStream;
use epl_common::database::entities::prelude::File;
use epl_common::options::{EplOptions, Options};
use epl_common::UploadedFileType;
use crate::AppState;

pub async fn get_attachment(
    Extension(state): Extension<AppState>,
    Path(query): Path<(i64, i64, String)>
) -> impl IntoResponse {
    let attachment_id = query.1;
    let filename = query.2;

    // TODO: Verify that the message still exists
    let file = File::find_by_id(attachment_id)
        .one(&state.conn)
        .await
        .expect("Unable to access database!");

    match file {
        None => {
            StatusCode::NOT_FOUND.into_response()
        }
        Some(file) => {
            if file.pending {
                return StatusCode::NOT_FOUND.into_response()
            }
            
            if file.r#type != UploadedFileType::Attachment as i32 {
                return StatusCode::NOT_FOUND.into_response()
            }

            if file.name != filename {
                return StatusCode::NOT_FOUND.into_response()
            }
            
            let object = state.aws
                .get_object()
                .bucket(EplOptions::get().s3_bucket)
                .key(format!("attachments/{}", file.id))
                .send()
                .await;

            match object {
                Ok(object) => {
                    let stream = ReaderStream::new(object.body.into_async_read());

                    let body = Body::from_stream(stream);

                    let headers = [
                        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file.name)),
                        (header::CONTENT_LENGTH, file.size.to_string()),
                        (header::CONTENT_RANGE, format!("bytes 0-{}/{}", (file.size - 1), file.size)),
                        (header::CONTENT_TYPE, file.content_type.unwrap_or("application/octet-stream".to_string())),
                        (header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Range, Accept-Ranges".to_string()),
                        (header::ACCEPT_RANGES, "bytes".to_string())
                    ];

                    (headers, body).into_response()
                }
                Err(_) => {
                    StatusCode::NOT_FOUND.into_response()
                }
            }
        }
    }
}