use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_derive::Deserialize;
use epl_common::database::entities::prelude::User;
use epl_common::database::entities::user;
use crate::AppState;
use crate::authorization_extractor::SessionContext;
use crate::nats::{RelationshipUpdate, send_relationship_update};

use sea_orm::prelude::*;
use crate::http::v9::errors::{APIErrorCode, throw_http_error};

#[derive(Deserialize)]
pub struct SendFriendRequestReq {
    username: String,
    discriminator: Option<u16>
}

pub async fn new_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Json(requested_user): Json<SendFriendRequestReq>
) -> impl IntoResponse {
    let normalized_discriminator: String = {
        if let Some(discriminator) = requested_user.discriminator {
            discriminator.to_string()
        } else {
            0.to_string()
        }
    };

    let requested_user: Option<user::Model> = User::find()
        .filter(user::Column::Username.eq(requested_user.username))
        .filter(user::Column::Discriminator.eq(normalized_discriminator))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_user {
        None => {
            StatusCode::NOT_FOUND.into_response()
        }
        Some(user) => {
            if user.id.eq(&session_context.user.id) {
                return (
                    StatusCode::BAD_REQUEST,
                    throw_http_error(
                        APIErrorCode::CannotSendFriendRequestToSelf,
                        vec![]
                    ).await
                ).into_response()
            }

            send_relationship_update(&state, user.id, session_context.user.id, RelationshipUpdate::Create).await;

            StatusCode::OK.into_response()
        }
    }
}

pub async fn delete_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>
) -> impl IntoResponse {
    let requested_user: Option<user::Model> = User::find_by_id(requested_user_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match requested_user {
        None => {
            StatusCode::BAD_REQUEST
        }
        Some(user) => {
            send_relationship_update(&state, user.id, session_context.user.id, RelationshipUpdate::Remove).await;

            StatusCode::NO_CONTENT
        }
    }
}