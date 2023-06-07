pub mod channels;
pub mod relationships;

use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use epl_common::database::entities::user;
use epl_common::flags::{generate_public_flags, get_user_flags, Badge};
use epl_common::Stub;
use sea_orm::EntityTrait;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ProfileRes {
    badges: Vec<Badge>,
    connected_accounts: Vec<ConnectedAccount>,
    // TODO: what is this?
    guild_badges: Vec<Stub>,
    legacy_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mutual_friends_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mutual_guilds: Option<Vec<Stub>>,
    premium_guild_since: Option<String>,
    premium_since: Option<String>,
    premium_type: i32,
    // TODO: should this be an option? or can we just leave this property out?
    profile_themes_experiment_bucket: Option<i32>,
    user: User,
    user_profile: UserProfile,
}

#[derive(Serialize)]
pub struct User {
    accent_color: Option<i32>,
    avatar: Option<String>,
    avatar_decoration: Option<String>,
    banner: Option<String>,
    banner_color: Option<String>,
    bio: String,
    discriminator: String,
    flags: i64,
    global_name: Option<String>,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_flags: Option<i64>,
    username: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    accent_color: Option<i32>,
    banner: Option<String>,
    bio: String,
    // TODO: guessing for string here
    emoji: Option<String>,
    // TODO: what
    popout_animation_particle_type: Option<String>,
    theme_colors: Vec<i32>,
}

#[derive(Serialize)]
pub struct ConnectedAccount {
    id: String,
    metadata: Option<ConnectedAccountMetadata>,
    name: String,
    _type: String,
    verified: bool,
}

#[derive(Serialize)]
pub enum ConnectedAccountMetadata {
    Ebay {
        created_at: String,
        positive_feedback_percentage: String,
        top_rated_seller: String,
        unique_negative_feedback_count: String,
        unique_positive_feedback_count: String,
        verified: String,
    },
    Steam {
        created_at: String,
        game_count: String,
        item_count_dota2: String,
        item_count_tf2: String,
    },
}

#[derive(Deserialize)]
pub struct ProfileQuery {
    with_mutual_guilds: bool,
    with_mutual_friends_count: bool,
}

pub async fn profile(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(requested_user_id): Path<i64>,
    profile_query: Query<ProfileQuery>,
) -> impl IntoResponse {
    let requested_user_opt: Option<user::Model> =
        epl_common::database::entities::prelude::User::find_by_id(requested_user_id)
            .one(&state.conn)
            .await
            .expect("Failed to access the database!");

    if requested_user_opt.is_none() {
        // TODO: check discord's status for invalid user ids
        return (StatusCode::NOT_FOUND).into_response();
    }

    let requested_user = requested_user_opt.unwrap();

    let flags = get_user_flags(requested_user.flags);

    let badges: Vec<Badge> = flags
        .iter()
        .filter_map(|i| {
            let x: Option<Badge> = (*i).into();
            x
        })
        .collect();

    // mostly stub
    let res = ProfileRes {
        badges,
        connected_accounts: vec![],
        guild_badges: vec![],
        legacy_username: None,
        mutual_friends_count: if profile_query.with_mutual_friends_count {
            Some(0)
        } else {
            None
        },
        mutual_guilds: if profile_query.with_mutual_guilds {
            Some(vec![])
        } else {
            None
        },
        premium_guild_since: None,
        premium_since: None,
        premium_type: requested_user.premium_type.unwrap_or(0),
        profile_themes_experiment_bucket: None,
        user: User {
            accent_color: None,
            avatar: requested_user.avatar,
            avatar_decoration: requested_user.avatar_decoration,
            banner: requested_user.banner.clone(),
            banner_color: requested_user.banner_colour,
            bio: requested_user.bio.clone().unwrap_or(String::new()),
            discriminator: requested_user.discriminator,
            flags: {
                if session_context.user.id.eq(&requested_user_id) {
                    requested_user.flags
                } else {
                    generate_public_flags(flags.clone())
                }
            },
            // FIXME: grab this when pomelo is impl
            global_name: None,
            id: requested_user.id.to_string(),
            public_flags: {
                if session_context.user.id.eq(&requested_user_id) {
                    Some(generate_public_flags(flags))
                } else {
                    None
                }
            },
            username: requested_user.username,
        },
        user_profile: UserProfile {
            accent_color: None,
            banner: requested_user.banner,
            bio: requested_user.bio.unwrap_or(String::new()),
            emoji: None,
            popout_animation_particle_type: None,
            theme_colors: vec![],
        },
    };

    (StatusCode::OK, Json(res)).into_response()
}
