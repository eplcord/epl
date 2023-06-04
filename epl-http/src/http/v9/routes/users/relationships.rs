use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, IntoActiveModel};
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::prelude::{Relationship, User};
use epl_common::database::entities::{relationship, user};
use crate::AppState;
use crate::authorization_extractor::SessionContext;
use crate::nats::{RelationshipUpdate, send_relationship_update};

use sea_orm::prelude::*;
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::RelationshipType;
use crate::http::v9::errors::{APIErrorCode, throw_http_error};

#[derive(Serialize)]
pub struct RelationshipResUser {
    avatar: Option<String>,
    avatar_decoration: Option<String>,
    discriminator: Option<String>,
    global_name: Option<String>,
    id: String,
    public_flags: i64,
    username: String
}

#[derive(Serialize)]
pub struct RelationshipRes {
    id: String,
    nickname: Option<String>,
    since: String,
    #[serde(rename = "type")]
    _type: i32,
    user: RelationshipResUser
}

pub async fn get_all_relationships(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let mut output: Vec<RelationshipRes> = vec![];

    let created_relationships = Relationship::find()
        .filter(relationship::Column::Creator.eq(session_context.user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let peered_relationships = Relationship::find()
        .filter(relationship::Column::Peer.eq(session_context.user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    // Gather created relationships first
    for i in created_relationships {
        let user: user::Model = User::find_by_id(i.peer)
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .expect("Missing user in relationship!");

        output.push(RelationshipRes {
            id: user.id.to_string(),
            nickname: None,
            since: i.timestamp.to_string(),
            _type: i.relationship_type,
            user: RelationshipResUser {
                avatar: user.avatar,
                avatar_decoration: user.avatar_decoration,
                discriminator: Some(user.discriminator),
                global_name: None,
                id: user.id.to_string(),
                public_flags: generate_public_flags(get_user_flags(user.flags)),
                username: user.username,
            },
        })
    }

    // Then peered relationships
    for i in peered_relationships {
        let user: user::Model = User::find_by_id(i.creator)
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .expect("Missing user in relationship!");

        output.push(RelationshipRes {
            id: user.id.to_string(),
            nickname: None,
            since: i.timestamp.to_string(),
            _type: i.relationship_type,
            user: RelationshipResUser {
                avatar: user.avatar,
                avatar_decoration: user.avatar_decoration,
                discriminator: Some(user.discriminator),
                global_name: None,
                id: user.id.to_string(),
                public_flags: generate_public_flags(get_user_flags(user.flags)),
                username: user.username,
            },
        })
    }

    Json(output)
}

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

            // Check if a relationship already exists
            let relationship_model = get_relationship(session_context.user.id, user.id, &state).await;

            if relationship_model.is_some() {
                return (
                    StatusCode::BAD_REQUEST,
                    throw_http_error(
                        APIErrorCode::FriendRequestBlocked,
                        vec![]
                    ).await
                ).into_response()
            }

            let new_relationship = relationship::ActiveModel {
                creator: Set(session_context.user.id),
                peer: Set(user.id),
                relationship_type: Set(RelationshipType::Outgoing as i32),
                timestamp: Set(chrono::Utc::now().naive_utc()),
            };

            match Relationship::insert(new_relationship).exec(&state.conn).await {
                Ok(_) => {
                    send_relationship_update(&state, user.id, session_context.user.id, RelationshipUpdate::Create).await;

                    StatusCode::OK.into_response()
                }
                Err(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }
}

// TODO: Move me to be shared across all API versions
pub async fn get_relationship(user_a: i64, user_b: i64, state: &AppState) -> Option<relationship::Model> {
    Relationship::find()
        .filter(
            Condition::any()
                .add(relationship::Column::Creator.eq(user_a))
                .add(relationship::Column::Creator.eq(user_b))
        )
        .filter(
            Condition::any()
                .add(relationship::Column::Peer.eq(user_a))
                .add(relationship::Column::Peer.eq(user_b))
        )
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
}

pub async fn delete_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>
) -> impl IntoResponse {
    let relationship_model = get_relationship(session_context.user.id, requested_user_id, &state).await;

    match relationship_model {
        None => {
            StatusCode::BAD_REQUEST
        }
        Some(relationship) => {
            let cached_relationship = (relationship.creator, relationship.peer);
            relationship.into_active_model()
                .delete(&state.conn)
                .await
                .expect("Failed to delete relationship!");

            send_relationship_update(&state, cached_relationship.0, cached_relationship.1, RelationshipUpdate::Remove).await;

            StatusCode::NO_CONTENT
        }
    }
}

#[derive(Deserialize)]
pub struct ModifyRelationshipReq {
    #[serde(rename = "type")]
    _type: Option<i32>
}

pub async fn modify_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>,
    Json(modify_relationship_req): Json<ModifyRelationshipReq>,
) -> impl IntoResponse {
    match modify_relationship_req._type.is_some() {
        true => {
            // We're probably blocking a user, lets see if a relationship already exists
            let relationship_model = get_relationship(session_context.user.id, requested_user_id, &state).await;

            if let Some(relationship) = relationship_model {
                let cached_relationship = (relationship.creator, relationship.peer);

                relationship.into_active_model().delete(&state.conn).await.expect("Failed to access database!");

                send_relationship_update(&state, cached_relationship.0, cached_relationship.1, RelationshipUpdate::Remove).await;
            }

            let new_relationship = relationship::ActiveModel {
                creator: Set(session_context.user.id),
                peer: Set(requested_user_id),
                relationship_type: Set(RelationshipType::Blocked.into()),
                timestamp: Set(chrono::Utc::now().naive_utc()),
            };

            match Relationship::insert(new_relationship).exec(&state.conn).await {
                Ok(_) => {
                    send_relationship_update(&state, session_context.user.id, requested_user_id, RelationshipUpdate::Block).await;

                    StatusCode::NO_CONTENT
                }
                Err(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        false => {
            // Accepting a friend request, lets modify it here
            let relationship_model = get_relationship(session_context.user.id, requested_user_id, &state).await;

            match relationship_model {
                None => {
                    StatusCode::BAD_REQUEST
                }
                Some(relationship) => {
                    let cached_relationship = (relationship.creator, relationship.peer);

                    let mut new_relationship = relationship.into_active_model();
                    new_relationship.relationship_type = Set(RelationshipType::Friend.into());
                    new_relationship.update(&state.conn).await.expect("Failed to access database!");

                    send_relationship_update(&state, cached_relationship.0, cached_relationship.1, RelationshipUpdate::Accept).await;

                    StatusCode::NO_CONTENT
                }
            }
        }
    }
}