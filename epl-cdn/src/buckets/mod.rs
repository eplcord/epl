mod avatars;
mod channel_icons;
mod badge_icons;

use aws_sdk_s3::primitives::ByteStream;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use ril::{Image, ResizeAlgorithm, Rgba};
use ril::ImageFormat::WebP;
use tokio_util::io::ReaderStream;
use tracing::error;
use epl_common::options::{EplOptions, Options};
use crate::AppState;
use crate::buckets::avatars::avatars;
use crate::buckets::badge_icons::badge_icons;
use crate::buckets::channel_icons::channel_icons;

pub fn buckets() -> Router {
    Router::new()
        // User Avatars
        .route("/avatars/:user_id/:file", get(avatars))
        // Channel Icons
        .route("/channel-icons/:channel_id/:file", get(channel_icons))
        // Badge Icons
        .route("/badge-icons/:file", get(badge_icons))
}

async fn get_image_or(
    bucket: &str,
    state: &AppState,
    user_id: u64,
    file: String,
) -> Result<impl IntoResponse, String> {
    let options = EplOptions::get();
    
    let object = state.aws
        .get_object()
        .bucket(options.s3_bucket)
        .key(format!("{bucket}/{user_id}/{file}"))
        .send()
        .await;

    match object {
        Ok(object) => {
            let stream = ReaderStream::new(object.body.into_async_read());

            let body = Body::from_stream(stream);

            let headers = [
                // TODO: detect this and fix this :)
                (header::CONTENT_TYPE, "image/webp"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{file}\"")),
            ];

            Ok((headers, body).into_response())
        }
        Err(e) => {
            Err(format!("{e}"))
        }
    }
}

pub async fn query_cached_size_or_create(
    bucket: &str,
    state: &AppState,
    user_id: u64,
    file: String,
    size: Option<u32>,
) -> impl IntoResponse {
    let options = EplOptions::get();
    
    match size {
        None => {
            match get_image_or(bucket, &state, user_id, file).await {
                Ok(v) => v.into_response(),
                Err(error) => {
                    error!("{error}");
                    StatusCode::NOT_FOUND.into_response()
                }
            }
        }
        Some(size) => {
            // Cap resizing images
            if !(16..=4096).contains(&size) {
                return StatusCode::BAD_REQUEST.into_response()
            }

            match get_image_or(bucket, &state, user_id, format!("{file}.{size}")).await {
                Ok(v) => v.into_response(),
                Err(_) => {
                    // Grab original image
                    let layer_two = state.aws
                        .get_object()
                        .bucket(options.s3_bucket.clone())
                        .key(format!("{bucket}/{user_id}/{file}"))
                        .send()
                        .await;

                    match layer_two {
                        Ok(layer_two) => {
                            let original_image_buffer = layer_two.body.collect().await.expect("Failed to read bytes!").to_vec();
                            let mut image_buffer: Vec<u8> = Vec::new();

                            // Resize the image
                            let mut image: Image<Rgba> = Image::from_bytes_inferred(original_image_buffer).expect("Invalid image!");
                            image.resize(size, size, ResizeAlgorithm::Nearest);
                            image.encode(WebP, &mut image_buffer).expect("Failed to encode image!");

                            let upload = state.aws.put_object()
                                .bucket(options.s3_bucket)
                                .key(format!("{bucket}/{user_id}/{file}.{size}"))
                                .body(ByteStream::from(image_buffer.clone()))
                                .send()
                                .await;

                            match upload {
                                Ok(_) => {
                                    let body = image_buffer;

                                    let headers = [
                                        // TODO: detect this and fix this :)
                                        (header::CONTENT_TYPE, "image/webp"),
                                        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{file}\"")),
                                    ];

                                    (headers, body).into_response()
                                }
                                Err(error) => {
                                    error!("{error}");
                                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                                }
                            }
                        }
                        Err(error) => {
                            error!("{error}");
                            StatusCode::NOT_FOUND.into_response()
                        }
                    }
                }
            }
        }
    }
}