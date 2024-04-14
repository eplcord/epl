use askama_axum::IntoResponse;
use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde_derive::{Deserialize, Serialize};
use tracing::error;
use epl_common::database::entities::file;
use epl_common::database::entities::prelude::{Channel, File};
use epl_common::{gen_token, UploadedFileType};
use epl_common::options::{EplOptions, Options};
use epl_common::permissions::{internal_permission_calculator, InternalChannelPermissions};
use epl_common::rustflake::Snowflake;
use crate::AppState;
use crate::authorization_extractor::SessionContext;

#[derive(Deserialize)]
pub struct PrepareS3Req {
    files: Vec<S3ReqFile>
}

#[derive(Deserialize)]
pub struct S3ReqFile {
    file_size: i64,
    filename: String,
    id: String,
    is_clip: Option<bool>
}

#[derive(Serialize)]
pub struct PrepareS3Res {
    attachments: Vec<S3ResAttachment>
}

#[derive(Serialize)]
pub struct S3ResAttachment {
    id: i64,
    upload_url: String,
    upload_filename: String
}

pub async fn prepare_s3_attachment_upload (
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(channel_id): Path<i64>,
    Json(data): Json<PrepareS3Req>
) -> impl IntoResponse {
    // Ensure channel actually exists
    let requested_channel = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_channel {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_channel) => {
            let calculated_permissions = internal_permission_calculator(
                &requested_channel,
                &session_context.user,
                None,
                &state.conn
            ).await;

            if !calculated_permissions.contains(&InternalChannelPermissions::AttachFiles) {
                return StatusCode::BAD_REQUEST.into_response()
            }

            let options = EplOptions::get();

            let mut res_vec: Vec<S3ResAttachment> = vec![];

            for i in data.files {
                let snowflake = Snowflake::default().generate();
                let upload_token = gen_token();

                let new_file = file::ActiveModel {
                    id: Set(snowflake),
                    upload_id: Set(Some(upload_token)),
                    pending: Set(true),
                    size: Set(i.file_size),
                    name: Set(i.filename),
                    timestamp: Set(chrono::Utc::now().naive_utc()),
                    r#type: Set(UploadedFileType::Attachment as i32),
                    uploader: Set(session_context.user.id),
                    ..Default::default()
                };

                match new_file.insert(&state.conn).await {
                    Ok(new_file) => {
                        res_vec.push(S3ResAttachment {
                            id: i.id.parse().unwrap_or_default(),
                            upload_url: format!("{}://{}/upload/{}/{}",
                                if options.require_ssl { "https" } else { "http" },
                                options.cdn_url,
                                new_file.upload_id.unwrap(),
                                new_file.name
                            ),
                            upload_filename: format!("{}", new_file.id),
                        });
                    }
                    Err(e) => {
                        error!("{:?}", e);

                        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }

            Json(PrepareS3Res { attachments: res_vec }).into_response()
        }
    }
}

pub async fn delete_attachment_upload(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(attachment_id): Path<i64>
) -> impl IntoResponse {
    let requested_attachment = File::find_by_id(attachment_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_attachment {
        None => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Some(requested_attachment) => {
            if !requested_attachment.uploader.eq(&session_context.user.id) {
                return StatusCode::BAD_REQUEST.into_response();
            }
            
            let mut attachment = requested_attachment.into_active_model();
            
            attachment.requested_deletion = Set(true);
            
            attachment.update(&state.conn).await.expect("Failed to access database!");

            StatusCode::NO_CONTENT.into_response()
        }
    }
}