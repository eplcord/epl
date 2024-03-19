use chrono::NaiveDateTime;
use sea_orm::EntityTrait;
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::note;
use epl_common::database::entities::prelude::Note;
use crate::AppState;
use crate::gateway::dispatch::{assemble_dispatch, DispatchTypes, send_message};
use crate::state::ThreadData;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserNoteUpdate {
    pub id: String,
    pub note: String,
}

pub async fn dispatch_user_note_update(
    thread_data: &mut ThreadData,
    state: &AppState,
    creator_id: i64,
    subject_id: i64,
) {
    let note: Option<note::Model> = Note::find_by_id((creator_id, subject_id)).one(&state.conn).await.expect("Failed to access database!");
    
    if let Some(note) = note {
        send_message(
            thread_data,
            assemble_dispatch(
                DispatchTypes::UserNoteUpdate(
                    UserNoteUpdate {
                        id: note.subject.to_string(),
                        note: note.text,
                    }
                )
            ),
        ).await;
    }
}