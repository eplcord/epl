use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::EntityTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::OnConflict;
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::note;
use epl_common::database::entities::prelude::Note;
use epl_common::nats::Messages::UserNoteUpdate;
use crate::AppState;
use crate::authorization_extractor::SessionContext;
use crate::http::v9::errors::{APIErrorCode, throw_http_error};
use epl_common::nats::send_nats_message;

#[derive(Serialize)]
pub struct GetNoteRes {
    user_id: String,
    note_user_id: String,
    note: String
}

pub async fn get_notes(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>
) -> impl IntoResponse {
    let note: Option<note::Model> = Note::find_by_id((session_context.user.id, requested_user_id)).one(&state.conn).await.expect("Failed to access database!");

    match note {
        None => {
            (StatusCode::NOT_FOUND, throw_http_error(
                APIErrorCode::UnknownUser,
                vec![]
            ).await).into_response()
        }
        Some(note) => {
            Json(GetNoteRes {
                user_id: note.creator.to_string(),
                note_user_id: note.subject.to_string(),
                note: note.text,
            }).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct PutNoteReq {
    note: String
}

pub async fn put_notes(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>,
    Json(put_note_req): Json<PutNoteReq>,
) -> impl IntoResponse {
    if put_note_req.note.len() > 256 {
        return StatusCode::BAD_REQUEST;
    }

    let note_struct = note::ActiveModel {
        creator: Set(session_context.user.id),
        subject: Set(requested_user_id),
        text: Set(put_note_req.note),
    };

    let update = note::Entity::insert(note_struct)
        .on_conflict(
            OnConflict::columns([note::Column::Creator, note::Column::Subject])
                .update_column(note::Column::Text)
                .to_owned()
        )
        .exec(&state.conn)
        .await;

    match update {
        Ok(_) => {
            send_nats_message(&state.nats_client, 
                              session_context.user.id.to_string(), 
                              UserNoteUpdate { 
                                  creator_id: session_context.user.id, 
                                  subject_id: requested_user_id
                              }).await;
            
            StatusCode::NO_CONTENT
        }
        Err(_) => {
            StatusCode::NOT_FOUND
        }
    }
}