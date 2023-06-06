use epl_common::database::entities::user;
use crate::gateway::dispatch::{assemble_dispatch, DispatchTypes, send_message};
use crate::gateway::schema::relationships::{RelationshipAdd, RelationshipRemove};
use crate::state::ThreadData;

use sea_orm::prelude::*;
use epl_common::database::entities::prelude::User;
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::RelationshipType;
use crate::AppState;
use crate::gateway::schema::SharedUser;

pub async fn dispatch_relationship_add(thread_data: &mut ThreadData, state: &AppState, originator: i64, req_type: RelationshipType) {
    let originating_user: user::Model = User::find_by_id(originator)
        .one(&state.conn)
        .await
        .expect("Failed to connect to database!")
        .expect("User missing!");

    send_message(thread_data, assemble_dispatch(
        DispatchTypes::RelationshipAdd(RelationshipAdd {
            id: originating_user.id.clone().to_string(),
            nickname: None,
            should_notify: true,
            since: chrono::Utc::now().to_string(),
            _type: req_type as i32,
            user: SharedUser {
                avatar: originating_user.avatar,
                avatar_decoration: originating_user.avatar_decoration,
                discriminator: Some(originating_user.discriminator),
                global_name: None,
                id: originating_user.id.clone().to_string(),
                public_flags: generate_public_flags(get_user_flags(originating_user.flags)),
                username: originating_user.username,
            },
        }),
    )).await;
}

pub async fn dispatch_relationship_remove(thread_data: &mut ThreadData, state: &AppState, originator: i64, req_type: RelationshipType) {
    let originating_user: user::Model = User::find_by_id(originator)
        .one(&state.conn)
        .await
        .expect("Failed to connect to database!")
        .expect("User missing!");

    send_message(thread_data, assemble_dispatch(
        DispatchTypes::RelationshipRemove(RelationshipRemove {
            id: originating_user.id.clone().to_string(),
            nickname: None,
            since: chrono::Utc::now().to_string(),
            _type: req_type as i32,
        }),
    )).await;
}