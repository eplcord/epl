use std::io;
use std::io::BufRead;

use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_s3::primitives::ByteStream;
use axum::body::{Body, StreamBody};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use ril::ImageFormat::WebP;
use ril::prelude::*;
use serde_derive::Deserialize;
use tokio_util::io::ReaderStream;
use tracing::error;
use tracing::log::debug;

use crate::AppState;

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

    match path_query.size {
        None => {
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
        Some(size) => {
            // Cap resizing images
            if size > 1024 || size == 0 {
                return StatusCode::BAD_REQUEST.into_response()
            }
            
            let layer_one = state.aws
                .get_object()
                .bucket("avatars")
                .key(format!("{user_id}/{file}.{size}"))
                .send()
                .await;

            match layer_one {
                Ok(layer_one) => {
                    let stream = ReaderStream::new(layer_one.body.into_async_read());

                    let body = StreamBody::new(stream);

                    let headers = [
                        // TODO: detect this and fix this :)
                        (header::CONTENT_TYPE, "image/webp"),
                        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{file}\"")),
                    ];

                    (headers, body).into_response()
                }
                Err(_) => {
                    // Grab original image
                    let layer_two = state.aws
                        .get_object()
                        .bucket("avatars")
                        .key(format!("{user_id}/{file}"))
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
                                .bucket("avatars")
                                .key(format!("{user_id}/{file}.webp.{size}"))
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