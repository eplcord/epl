use crate::authorization_extractor::SessionContext;
use crate::nats::{RelationshipUpdate, send_relationship_update};
use crate::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use epl_common::database::entities::prelude::{Relationship, User};
use epl_common::database::entities::{relationship, user};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde_derive::{Deserialize, Serialize};

use crate::http::v9::errors::{APIErrorCode, throw_http_error};
use epl_common::RelationshipType;
use sea_orm::prelude::*;
use epl_common::relationship::get_relationship;
use epl_common::schema::v9;
use epl_common::schema::v9::user::generate_user_struct;

#[derive(Serialize)]
pub struct RelationshipRes {
    id: String,
    nickname: Option<String>,
    since: String,
    #[serde(rename = "type")]
    _type: i32,
    user: v9::user::User,
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
        .filter(relationship::Column::RelationshipType.ne(RelationshipType::Blocked as i32))
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
            since: i.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            _type: i.relationship_type,
            user: generate_user_struct(user),
        })
    }

    // Then peered relationships
    for i in peered_relationships {
        let user: user::Model = User::find_by_id(i.creator)
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .expect("Missing user in relationship!");

        // For peered relationships, they will show as outgoing when the peer should see it as incoming
        let normalized_type = if i.relationship_type == (RelationshipType::Outgoing as i32) {
            RelationshipType::Incoming as i32
        } else {
            i.relationship_type
        };

        output.push(RelationshipRes {
            id: user.id.to_string(),
            nickname: None,
            since: i.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            _type: normalized_type,
            user: generate_user_struct(user),
        })
    }

    Json(output)
}

#[derive(Deserialize)]
pub struct SendFriendRequestReq {
    username: String,
    discriminator: Option<u16>,
}

pub async fn new_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Json(requested_user): Json<SendFriendRequestReq>,
) -> impl IntoResponse {
    let normalized_discriminator: String = {
        if let Some(discriminator) = requested_user.discriminator {
            let mut output = discriminator.to_string();

            while output.chars().count() < 4 {
                output.insert(0, '0');
            }

            output
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
        None => StatusCode::NOT_FOUND.into_response(),
        Some(user) => {
            if user.id.eq(&session_context.user.id) {
                return (
                    StatusCode::BAD_REQUEST,
                    throw_http_error(APIErrorCode::CannotSendFriendRequestToSelf, vec![]).await,
                )
                    .into_response();
            }

            // Check if a relationship already exists
            let relationship_model =
                get_relationship(session_context.user.id, user.id, &state.conn).await;

            if relationship_model.is_some() {
                return (
                    StatusCode::BAD_REQUEST,
                    throw_http_error(APIErrorCode::FriendRequestBlocked, vec![]).await,
                )
                    .into_response();
            }

            let new_relationship = relationship::ActiveModel {
                creator: Set(session_context.user.id),
                peer: Set(user.id),
                relationship_type: Set(RelationshipType::Outgoing as i32),
                timestamp: Set(chrono::Utc::now().naive_utc()),
            };

            match Relationship::insert(new_relationship)
                .exec(&state.conn)
                .await
            {
                Ok(_) => {
                    send_relationship_update(
                        &state,
                        user.id,
                        session_context.user.id,
                        RelationshipUpdate::Create,
                    )
                    .await;

                    StatusCode::OK.into_response()
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
}

pub async fn delete_relationship(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>,
) -> impl IntoResponse {
    let relationship_model =
        get_relationship(session_context.user.id, requested_user_id, &state.conn).await;

    match relationship_model {
        None => StatusCode::BAD_REQUEST,
        Some(relationship) => {
            let cached_relationship = (relationship.creator, relationship.peer);
            relationship
                .into_active_model()
                .delete(&state.conn)
                .await
                .expect("Failed to delete relationship!");

            send_relationship_update(
                &state,
                cached_relationship.0,
                cached_relationship.1,
                RelationshipUpdate::Remove,
            )
            .await;

            StatusCode::NO_CONTENT
        }
    }
}

#[derive(Deserialize)]
pub struct ModifyRelationshipReq {
    #[serde(rename = "type")]
    _type: Option<i32>,
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
            let relationship_model =
                get_relationship(session_context.user.id, requested_user_id, &state.conn).await;

            if let Some(relationship) = relationship_model {
                let cached_relationship = (relationship.creator, relationship.peer);

                relationship
                    .into_active_model()
                    .delete(&state.conn)
                    .await
                    .expect("Failed to access database!");

                send_relationship_update(
                    &state,
                    cached_relationship.0,
                    cached_relationship.1,
                    RelationshipUpdate::Remove,
                )
                .await;
            }

            let new_relationship = relationship::ActiveModel {
                creator: Set(session_context.user.id),
                peer: Set(requested_user_id),
                relationship_type: Set(RelationshipType::Blocked as i32),
                timestamp: Set(chrono::Utc::now().naive_utc()),
            };

            match Relationship::insert(new_relationship)
                .exec(&state.conn)
                .await
            {
                Ok(_) => {
                    send_relationship_update(
                        &state,
                        session_context.user.id,
                        requested_user_id,
                        RelationshipUpdate::Block,
                    )
                    .await;

                    StatusCode::NO_CONTENT.into_response()
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        false => {
            // Let's see if we're accepting a friend request or sending a new friend request directly (skipping new_relationship)
            let relationship_model =
                get_relationship(session_context.user.id, requested_user_id, &state.conn).await;

            match relationship_model {
                None => {
                    // Don't allow users to friend themselves
                    if requested_user_id.eq(&session_context.user.id) {
                        return (
                            StatusCode::BAD_REQUEST,
                            throw_http_error(APIErrorCode::CannotSendFriendRequestToSelf, vec![])
                                .await,
                        )
                            .into_response();
                    };

                    let new_relationship = relationship::ActiveModel {
                        creator: Set(session_context.user.id),
                        peer: Set(requested_user_id),
                        relationship_type: Set(RelationshipType::Outgoing as i32),
                        timestamp: Set(chrono::Utc::now().naive_utc()),
                    };

                    match Relationship::insert(new_relationship)
                        .exec(&state.conn)
                        .await
                    {
                        Ok(_) => {
                            send_relationship_update(
                                &state,
                                requested_user_id,
                                session_context.user.id,
                                RelationshipUpdate::Create,
                            )
                            .await;

                            StatusCode::NO_CONTENT.into_response()
                        }
                        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    }
                }
                Some(relationship) => {
                    let cached_relationship = (relationship.creator, relationship.peer);

                    let mut new_relationship = relationship.into_active_model();
                    new_relationship.relationship_type = Set(RelationshipType::Friend as i32);
                    new_relationship
                        .update(&state.conn)
                        .await
                        .expect("Failed to access database!");

                    send_relationship_update(
                        &state,
                        cached_relationship.0,
                        cached_relationship.1,
                        RelationshipUpdate::Accept,
                    )
                    .await;

                    StatusCode::NO_CONTENT.into_response()
                }
            }
        }
    }
}
