use std::io::Cursor;
use aws_sdk_s3::primitives::ByteStream;
use axum::body::Bytes;
use axum::Extension;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use mp4::TrackType;
use ril::{Image, Rgba};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use sea_orm::ActiveValue::Set;
use tracing::error;
use epl_common::database::entities::file;
use epl_common::database::entities::prelude::File;
use epl_common::options::{EplOptions, Options};
use crate::AppState;

pub async fn upload_attachment(
    Extension(state): Extension<AppState>,
    Path(query): Path<(String, String)>,
    body: Bytes,
) -> impl IntoResponse {
    let file_upload_key = query.0;
    let filename = query.1;

    // Find the requested file slot in the database
    let file_slot = File::find()
        .filter(file::Column::UploadId.eq(file_upload_key))
        .one(&state.conn)
        .await
        .expect("Failed to access the database!");

    match file_slot {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(file) => {
            if filename != file.name {
                return StatusCode::BAD_REQUEST.into_response()
            }

            if body.len() != file.size as usize {
                return StatusCode::BAD_REQUEST.into_response()
            }

            // So this part will fail to build if it's not surrounded by brackets which is why its
            // like this. Idk why, im guessing cookie leaks something and this causes cookie to be
            // dropped early after we're done with it
            let kind = {
                let cookie = magic::Cookie::open(magic::cookie::Flags::ERROR | magic::cookie::Flags::MIME_TYPE)
                    .expect("Failed to open magic cookie!")
                    .load(&Default::default())
                    .expect("Failed to open magic database!");

                cookie.buffer(&body).ok()
            };

            let mut width: Option<i64> = None;
            let mut height: Option<i64> = None;

            if kind.clone().is_some_and(|x| matches!(x.as_str(), "image/gif" | "image/png" | "image/jpeg" | "image/webp" | "image/x-icon")) {
                let image: Image<Rgba> = Image::from_bytes_inferred(body.clone()).expect("Invalid image!");

                width = Some(image.width() as i64);
                height = Some(image.height() as i64);
            } else if kind.clone().is_some_and(|x| x.as_str() == "video/mp4") {
                let mp4 = mp4::Mp4Reader::read_header(Cursor::new(body.clone()), body.len() as u64).expect("Invalid mp4 file!");
            
                for i in mp4.tracks().values() {
                    if i.track_type().expect("what") == TrackType::Video {
                        // ok we hit our first video track, lets just use this
                        width = Some(i.width() as i64);
                        height = Some(i.height() as i64);
            
                        break;
                    }
                }
            } else if kind.clone().is_some_and(|x| matches!(x.as_str(), "video/webm")) {
                // TODO: idk something with like ffmpeg here
            }

            let s3_res = state.aws.put_object()
                .bucket(EplOptions::get().s3_bucket)
                .key(format!("attachments/{}", file.id))
                .body(ByteStream::from(body))
                .send()
                .await;

            match s3_res {
                Ok(_) => {
                    let mut active_file = file.into_active_model();

                    active_file.content_type = Set(kind);
                    active_file.pending = Set(false);

                    active_file.width = Set(width);
                    active_file.height = Set(height);

                    // TODO: figure out width and height for support file types :3
                    
                    match active_file.update(&state.conn).await {
                        Ok(_) => {
                            StatusCode::OK.into_response()
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            
                            StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("{:?}", e);

                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }
}