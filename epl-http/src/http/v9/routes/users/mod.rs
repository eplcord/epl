pub mod channels;
pub mod relationships;
pub mod notes;

use std::io;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use aws_sdk_s3::primitives::ByteStream;
use crate::authorization_extractor::SessionContext;
use crate::AppState;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use epl_common::database::entities::{session, user};
use epl_common::flags::{generate_public_flags, get_user_flags, Badge, UserFlags};
use epl_common::Stub;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use sea_orm::ActiveValue::Set;
use serde_derive::{Deserialize, Serialize};
use epl_common::nats::Messages;
use epl_common::nats::send_nats_message;
use base64::prelude::*;
use ril::ImageFormat::WebP;
use ril::prelude::*;
use epl_common::options::{EplOptions, Options};

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
    bio: Option<String>,
    // TODO: guessing for string here
    emoji: Option<String>,
    // TODO: what
    popout_animation_particle_type: Option<String>,
    theme_colors: Option<Vec<i32>>,
    pronouns: Option<String>,
    profile_effect: Option<String>
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
    with_mutual_guilds: Option<bool>,
    with_mutual_friends_count: Option<bool>,
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

    let mut badges: Vec<Badge> = flags
        .iter()
        .filter_map(|i| {
            let x: Option<Badge> = (*i).into();
            x
        })
        .collect();
    
    if requested_user.legacy_name.is_some() {
        badges.push(Badge {
            description: format!("Originally known as {}", requested_user.legacy_name.clone().unwrap()),
            icon: "6de6d34650760ba5551a79732e98ed60".to_string(),
            id: "legacy_username".to_string(),
            link: None,
        });
    }

    // mostly stub
    let res = ProfileRes {
        badges,
        connected_accounts: vec![],
        guild_badges: vec![],
        legacy_username: requested_user.legacy_name,
        mutual_friends_count: if profile_query.with_mutual_friends_count.unwrap_or(false) {
            Some(0)
        } else {
            None
        },
        mutual_guilds: if profile_query.with_mutual_guilds.unwrap_or(false) {
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
            bio: requested_user.bio.clone().unwrap_or_default(),
            discriminator: requested_user.discriminator,
            flags: {
                if session_context.user.id.eq(&requested_user_id) {
                    requested_user.flags
                } else {
                    generate_public_flags(flags.clone())
                }
            },
            // FIXME: grab this when pomelo is impl
            global_name: requested_user.display_name.clone(),
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
            accent_color: requested_user.accent_color.map(|x| x.parse::<i32>().unwrap()),
            banner: requested_user.banner,
            bio: requested_user.bio,
            emoji: None,
            popout_animation_particle_type: None,
            theme_colors: None,
            pronouns: requested_user.pronouns,
            profile_effect: None,
        },
    };

    (StatusCode::OK, Json(res)).into_response()
}

#[derive(Deserialize)]
pub struct DisableReq {
    pub password: String,
}

pub async fn disable_account(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<DisableReq>,
) -> impl IntoResponse {
    // Verify password
    let password_hash =
        PasswordHash::new(&session_context.user.password_hash).expect("Failed to parse password hash!");
    
    match Argon2::default()
        .verify_password(data.password.as_bytes(), &password_hash) {
        Ok(_) => {
            let mut flags = session_context.user.flags;

            flags += UserFlags::Disabled as i64;

            let mut active_user = session_context.user.into_active_model();

            active_user.flags = Set(flags);

            match active_user.update(&state.conn).await {
                Ok(user) => {
                    send_nats_message(&state.nats_client, user.id.to_string(), Messages::InvalidateGatewaySession { session: "all".to_string() }).await;

                    let session_delete_res = session::Entity::delete_many()
                        .filter(session::Column::UserId.eq(user.id))
                        .exec(&state.conn)
                        .await;

                    match session_delete_res {
                        Ok(_) => {
                            StatusCode::NO_CONTENT
                        }
                        Err(_) => {
                            StatusCode::INTERNAL_SERVER_ERROR
                        }
                    }
                },
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => {
            StatusCode::BAD_REQUEST
        }
    }
}

pub async fn delete_account() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

#[derive(Deserialize, Clone)]
pub struct UpdateUserReq {
    pub avatar: Option<String>,
    pub global_name: Option<String>,
    pub new_password: Option<String>,
    pub password: Option<String>
}

pub async fn update_user(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<UpdateUserReq>,
) -> impl IntoResponse {
    let mut active_user = session_context.user.into_active_model();
    let options = EplOptions::get();

    if let Some(avatar) = &data.avatar {
        // TODO: Supports gifs
        let image_bytes = avatar.split("base64,").collect::<Vec<&str>>()[1].as_bytes();
        let image = BASE64_STANDARD.decode(image_bytes).expect("Invalid base64! Bailing!");

        let hash = sha256::digest(&image);

        let mut image_buffer: Vec<u8> = Vec::new();
        let image: Image<Rgba> = Image::from_reader_inferred(&mut io::Cursor::new(image)).expect("Invalid image!");
        image.encode(WebP, &mut image_buffer).expect("Failed to encode image!");

        let s3_res = state.aws.put_object()
            .bucket(options.s3_bucket)
            .key(format!("avatars/{}/{hash}.webp", active_user.clone().id.unwrap()))
            .body(ByteStream::from(image_buffer))
            .send()
            .await;

        match s3_res {
            Ok(_) => {
                active_user.avatar = Set(Some(hash.to_string()));
            }
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    if let Some(global_name) = &data.global_name {
        if global_name.is_empty() {
            active_user.display_name = Set(None);
        } else {
            active_user.display_name = Set(Some(global_name.clone()));
        }
    }

    if let Some(new_password) = &data.new_password {
        let data = data.clone();

        if data.password.is_none() {
            return StatusCode::BAD_REQUEST.into_response();
        }

        let current_password = data.password.clone().unwrap();
        let current_password_hash = active_user.password_hash.clone().unwrap();

        if current_password.is_empty() {
            return StatusCode::BAD_REQUEST.into_response();
        }

        // Verify password
        let password_hash =
            PasswordHash::new(&current_password_hash).expect("Failed to parse password hash!");

        match Argon2::default()
            .verify_password(current_password.as_bytes(), &password_hash) {
            Ok(_) => {
                // TODO: Implement :)
                return StatusCode::NOT_IMPLEMENTED.into_response();
            }
            Err(_) => {
                return StatusCode::BAD_REQUEST.into_response();
            }
        }
    }

    match active_user.update(&state.conn).await {
        Ok(user) => {
            Json(
                epl_common::User {
                    verified: user.acct_verified,
                    username: user.username,
                    purchased_flags: user.purchased_flags.unwrap_or(0),
                    premium_type: user.premium_type.unwrap_or(0),
                    premium: (user.premium_type.unwrap_or(0) != 0),
                    phone: user.phone,
                    nsfw_allowed: user.nsfw_allowed,
                    // FIXME: We need to store more information about the current session
                    mobile: false,
                    mfa_enabled: user.mfa_enabled,
                    id: user.id.to_string(),
                    // TODO: pomelo related?
                    global_name: user.display_name.clone(),
                    flags: user.flags,
                    email: user.email,
                    // TODO: pomelo related?
                    display_name: user.display_name,
                    discriminator: user.discriminator,
                    // FIXME: Same as "mobile"
                    desktop: false,
                    bio: user.bio.unwrap_or(String::new()),
                    banner_color: user.banner_colour,
                    banner: user.banner,
                    avatar_decoration: user.avatar_decoration,
                    avatar: user.avatar,
                    accent_color: user.accent_color,
                }
            ).into_response()
        }
        Err(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateProfileReq {
    pub accent_color: Option<i32>,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub profile_effect: Option<String>,
    pub banner: Option<String>,
    pub theme_colors: Option<String>,
    pub popout_animation_particle_type: Option<String>,
    pub emoji: Option<String>
}

pub async fn update_profile(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<UpdateProfileReq>,
) -> impl IntoResponse {
    let mut active_user = session_context.user.into_active_model();

    if let Some(accent_color) = data.accent_color {
        active_user.accent_color = Set(Some(accent_color.to_string()));
    }

    if let Some(bio) = &data.bio {
        active_user.bio = Set(Some(bio.clone()));
    }

    if let Some(pronouns) = &data.pronouns {
        active_user.pronouns = Set(Some(pronouns.clone()));
    }

    if let Some(banner) = &data.banner {
        active_user.banner = Set(Some(banner.clone()));
    }

    match active_user.update(&state.conn).await {
        Ok(user) => {
            Json(UserProfile {
                accent_color: user.accent_color.map(|x| x.parse::<i32>().unwrap()),
                bio: user.bio,
                pronouns: user.pronouns,
                banner: user.banner,
                // TODO: Figure these out
                profile_effect: None,
                theme_colors: None,
                popout_animation_particle_type: None,
                emoji: None,
            }).into_response()
        }
        Err(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct PomeloReq {
    pub username: String,
    pub global_name: Option<String>
}

pub async fn pomelo(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<PomeloReq>
) -> impl IntoResponse {
    let cached_legacy_username: (String, String) = (session_context.user.username.clone(), session_context.user.discriminator.clone());

    let mut active_user = session_context.user.into_active_model();

    let rename_check: Option<user::Model> = user::Entity::find()
        .filter(user::Column::Username.eq(data.username.clone()))
        .filter(user::Column::Discriminator.eq(0.to_string()))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    match rename_check {
        None => {
            active_user.username = Set(data.username.clone());
            active_user.discriminator = Set(0.to_string());

            active_user.legacy_name = Set(Some(format!("{}#{}", cached_legacy_username.0, cached_legacy_username.1)));

            active_user.display_name = Set(data.global_name.clone());

            match active_user.update(&state.conn).await {
                Ok(user) => {
                    Json(User {
                        accent_color: user.accent_color.map(|x| x.parse().unwrap()),
                        avatar: user.avatar,
                        avatar_decoration: user.avatar_decoration,
                        banner: user.banner,
                        banner_color: user.banner_colour,
                        bio: user.bio.unwrap_or_default(),
                        discriminator: user.discriminator,
                        flags: user.flags,
                        global_name: Some(user.username.clone()),
                        id: user.id.to_string(),
                        public_flags: Some(generate_public_flags(get_user_flags(user.flags))),
                        username: user.username,
                    }).into_response()
                }
                Err(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Some(_) => {
            // TODO: Figure out the correct error to return if the username is already used
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}