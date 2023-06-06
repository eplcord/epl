use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde_derive::{Deserialize, Serialize};
use epl_common::database::entities::prelude::{Channel, ChannelMember, User};
use epl_common::database::entities::{channel, channel_member, user};
use epl_common::rustflake::Snowflake;
use crate::AppState;
use crate::authorization_extractor::SessionContext;

use sea_orm::prelude::*;
use serde_with::skip_serializing_none;
use epl_common::channels::ChannelTypes;
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::nats::Messages::ChannelCreate;
use epl_common::RelationshipType;
use crate::http::v9::routes::users::relationships::get_relationship;
use crate::nats::send_nats_message;

#[derive(Deserialize)]
pub struct NewDMChannelReq {
    recipients: Vec<String>
}

#[derive(Serialize, Clone)]
pub struct NewDMChannelUser {
    pub accent_color: Option<i32>,
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub banner: Option<String>,
    pub banner_color: Option<String>,
    pub discriminator: Option<String>,
    pub display_name: Option<String>,
    pub flags: i64,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String
}

#[skip_serializing_none]
#[derive(Serialize, Clone)]
pub struct NewDMChannelRes {
    pub flags: i64,
    pub id: String,
    pub icon: Option<String>,
    #[serialize_always]
    pub last_message_id: Option<String>,
    pub name: Option<String>,
    pub owner_id: Option<String>,
    pub recipients: Vec<NewDMChannelUser>,
    #[serde(rename = "type")]
    pub _type: i32,
}

pub async fn new_dm_channel(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Json(new_channel_dm_req): Json<NewDMChannelReq>,
) -> impl IntoResponse {
    let snowflake = Snowflake::default().generate();

    let mut users: Vec<NewDMChannelUser> = vec![];
    let mut channel_members: Vec<channel_member::ActiveModel> = vec![];

    // First we ensure the users both exist and are friends with the creator
    for i in &new_channel_dm_req.recipients {
        let user: Option<user::Model> = User::find_by_id(i.parse::<i64>().expect("User ID is not i64!"))
            .one(&state.conn)
            .await
            .expect("Failed to access database!");

        match user {
            None => {
                return StatusCode::BAD_REQUEST.into_response()
            }
            Some(user) => {
                // User exists, now we check if they're actually friends
                match get_relationship(session_context.user.id, user.id, &state).await {
                    None => {
                        // They're not, bail
                        return StatusCode::BAD_REQUEST.into_response()
                    }
                    Some(relationship) => {
                        if relationship.relationship_type != RelationshipType::Friend as i32 {
                            // They're either pending or blocked, bail
                            return StatusCode::BAD_REQUEST.into_response()
                        } else {
                            // They're friends, keep going

                            // We'll also generate the channel member entries now
                            channel_members.push(channel_member::Model {
                                channel: snowflake,
                                user: user.id
                            }.into_active_model());

                            // And the required NewDMChannelUser for the HTTP response
                            users.push(NewDMChannelUser {
                                accent_color: user.accent_color.map(|e| e.parse().expect("Failed to parse user's accent_color")),
                                avatar: user.avatar,
                                avatar_decoration: user.avatar_decoration,
                                banner: user.banner,
                                banner_color: user.banner_colour,
                                discriminator: Some(user.discriminator),
                                display_name: None,
                                flags: generate_public_flags(get_user_flags(user.flags)),
                                global_name: None,
                                id: user.id.to_string(),
                                public_flags: generate_public_flags(get_user_flags(user.flags)),
                                username: user.username,
                            });
                        }
                    }
                }
            }
        }
    }

    // Now we do the same as above but for the creator
    channel_members.push(channel_member::Model {
        channel: snowflake,
        user: session_context.user.id
    }.into_active_model());

    // Calculate if we should insert a DM or group DM
    let mut channel_type = ChannelTypes::DM;

    if new_channel_dm_req.recipients.len() > 1 {
        channel_type = ChannelTypes::GroupDM;
    }

    // Now that we've verified that all the users are friends, create the channel
    Channel::insert(
        channel::ActiveModel {
            id: Set(snowflake),
            r#type: Set(channel_type as i32),
            owner_id: {
                match channel_type {
                    ChannelTypes::GroupDM => Set(Some(session_context.user.id)),
                    _ => Set(None)
                }
            },
            ..Default::default()
        }
    ).exec(&state.conn)
        .await
        .expect("Failed to access database!");

    // Insert the channel member entries now that the channel is made
    ChannelMember::insert_many(channel_members.clone())
        .exec(&state.conn)
        .await
        .expect("Failed to access the database!");

    // Now we inform everyone about the channel creation
    for i in channel_members {
        send_nats_message(&state.nats_client, i.user.unwrap().to_string(), ChannelCreate { id: snowflake }).await;
    }

    (StatusCode::OK, Json(NewDMChannelRes {
        flags: 0,
        id: snowflake.to_string(),
        icon: None,
        last_message_id: None,
        name: None,
        owner_id: {
            match channel_type {
                ChannelTypes::GroupDM => Some(session_context.user.id.to_string()),
                _ => None
            }
        },
        recipients: users,
        _type: channel_type as i32,
    })).into_response()
}