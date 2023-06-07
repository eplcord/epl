use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use epl_common::database::entities::user;
use epl_common::flags::{get_user_flags, UserFlags};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use serde_derive::Deserialize;

use crate::authorization_extractor::SessionContext;
use crate::AppState;

#[derive(Deserialize)]
pub struct HypesquadReq {
    pub house_id: i32,
}

pub async fn join_hypesquad(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<HypesquadReq>,
) -> impl IntoResponse {
    if !(1..3).contains(&data.house_id) {
        return StatusCode::BAD_REQUEST;
    }

    let mut final_flags = session_context.user.flags;

    for i in get_user_flags(session_context.user.flags) {
        match i {
            UserFlags::HypesquadOnlineHouse1 => {
                final_flags -= UserFlags::HypesquadOnlineHouse1 as i64;
            }
            UserFlags::HypesquadOnlineHouse2 => {
                final_flags -= UserFlags::HypesquadOnlineHouse2 as i64;
            }
            UserFlags::HypesquadOnlineHouse3 => {
                final_flags -= UserFlags::HypesquadOnlineHouse3 as i64;
            }
            _ => {}
        }
    }

    match data.house_id {
        1 => {
            final_flags += UserFlags::HypesquadOnlineHouse1 as i64;
        }
        2 => {
            final_flags += UserFlags::HypesquadOnlineHouse2 as i64;
        }
        3 => {
            final_flags += UserFlags::HypesquadOnlineHouse3 as i64;
        }
        _ => return StatusCode::BAD_REQUEST,
    }

    let mut updated_user: user::ActiveModel = session_context.user.into_active_model();

    updated_user.flags = Set(final_flags);

    match updated_user.update(&state.conn).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn leave_hypesquad(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let mut final_flags = session_context.user.flags;

    for i in get_user_flags(session_context.user.flags) {
        match i {
            UserFlags::HypesquadOnlineHouse1 => {
                final_flags -= UserFlags::HypesquadOnlineHouse1 as i64;
            }
            UserFlags::HypesquadOnlineHouse2 => {
                final_flags -= UserFlags::HypesquadOnlineHouse2 as i64;
            }
            UserFlags::HypesquadOnlineHouse3 => {
                final_flags -= UserFlags::HypesquadOnlineHouse3 as i64;
            }
            _ => {}
        }
    }

    let mut updated_user: user::ActiveModel = session_context.user.into_active_model();

    updated_user.flags = Set(final_flags);

    match updated_user.update(&state.conn).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
