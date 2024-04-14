mod auth;
mod channels;
mod hypesquad;
mod tracking;
mod users;
mod aprilfools2024;
mod gifs;

use crate::authorization_extractor::get_session_context;
use crate::http::v9::routes::auth::{
    location_metadata, login, logout, logout_session, register, sessions, verify_email,
};
use crate::http::v9::routes::channels::{add_user_to_channel, delete_message, edit_message, get_messages, modify_channel, remove_user_from_channel, send_message, typing};
use crate::http::v9::routes::hypesquad::{join_hypesquad, leave_hypesquad};
use crate::http::v9::routes::users::channels::new_dm_channel;
use crate::http::v9::routes::users::{disable_account, pomelo, profile, update_profile, update_user};
use crate::http::v9::routes::users::relationships::{
    delete_relationship, get_all_relationships, modify_relationship, new_relationship,
};
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, middleware, Router};
use axum::response::IntoResponse;
use serde_derive::Serialize;
use epl_common::Stub;
use crate::debug::debug_body;
use crate::http::v9::routes::aprilfools2024::{count_lootboxes, get_lootboxes, open_lootbox, redeem_prize};
use crate::http::v9::routes::channels::attachments::{delete_attachment_upload, prepare_s3_attachment_upload};
use crate::http::v9::routes::channels::pins::{delete_pin, get_pins, new_pin};
use crate::http::v9::routes::channels::reactions::{add_reaction, delete_specific_user_reaction, get_reactions, remove_reaction};
use crate::http::v9::routes::gifs::{actually_get_trending_gifs, get_trending_gifs, gif_search_suggestions, search_gifs};
use crate::http::v9::routes::tracking::science;
use crate::http::v9::routes::users::notes::{get_notes, put_notes};

pub fn assemble_routes() -> Router {
    let sessions = Router::new()
        .route("/", get(sessions))
        .route("/logout", post(logout_session));

    let authenticated_auth = Router::new()
        .route("/logout", post(logout))
        .route("/verify", post(verify_email))
        .route("/verify/resend", post(verify_email))
        .nest("/sessions", sessions)
        .route_layer(middleware::from_fn(get_session_context));

    let auth = Router::new()
        .route("/location-metadata", get(location_metadata))
        .route("/login", post(login))
        .route("/register", post(register))
        .merge(authenticated_auth);

    let atme = Router::new()
        .route("/relationships", get(get_all_relationships))
        .route("/relationships", post(new_relationship))
        .route("/relationships/:user_id", delete(delete_relationship))
        .route("/relationships/:user_id", put(modify_relationship))
        .route("/channels", post(new_dm_channel))
        .route("/disable", post(disable_account))
        .route("/profile", patch(update_profile))
        .route("/notes/:user_id", get(get_notes))
        .route("/notes/:user_id", put(put_notes))
        .route("/pomelo", post(pomelo))
        .route("/devices", post(science))
        .route("/lootboxes/open", post(open_lootbox))
        .route("/lootboxes/redeem-prize", post(redeem_prize))
        .route("/lootboxes", get(get_lootboxes))
        .route("/", patch(update_user));

    let users = Router::new()
        .nest("/@me", atme)
        .route("/:user_id/profile", get(profile))
        // Workaround
        .route("/%40me/profile", patch(update_profile))
        .route_layer(middleware::from_fn(get_session_context));

    let hypesquad = Router::new()
        .route("/online", post(join_hypesquad))
        .route("/online", delete(leave_hypesquad))
        .route_layer(middleware::from_fn(get_session_context));

    let channels = Router::new()
        .route("/:channel_id/messages/:message_id", patch(edit_message))
        .route("/:channel_id/messages/:message_id", delete(delete_message))
        .route("/:channel_id/messages/:message_id/reactions/:emoji", get(get_reactions))
        .route("/:channel_id/messages/:message_id/reactions/:emoji/%40me", put(add_reaction))
        .route("/:channel_id/messages/:message_id/reactions/:emoji/:type/%40me", delete(remove_reaction))
        .route("/:channel_id/messages/:message_id/reactions/:emoji/:type/:user_id", delete(delete_specific_user_reaction))
        .route("/:channel_id/messages", get(get_messages))
        .route("/:channel_id/messages", post(send_message))
        .route("/:channel_id/typing", post(typing))
        .route("/:channel_id/recipients/:user_id", put(add_user_to_channel))
        .route("/:channel_id/recipients/:user_id", delete(remove_user_from_channel))
        .route("/:channel_id/pins", get(get_pins))
        .route("/:channel_id/pins/:message_id", put(new_pin))
        .route("/:channel_id/pins/:message_id", delete(delete_pin))
        .route("/:channel_id/attachments", post(prepare_s3_attachment_upload))
        .route("/:channel_id", patch(modify_channel))
        .route_layer(middleware::from_fn(get_session_context));

    let gifs = Router::new()
        .route("/search", get(search_gifs))
        .route("/trending", get(get_trending_gifs))
        .route("/trending-gifs", get(actually_get_trending_gifs))
        .route("/suggest", get(gif_search_suggestions))
        .route_layer(middleware::from_fn(get_session_context));

    let safetyhub = Router::new()
        .route("/@me", get(account_standing))
        .route_layer(middleware::from_fn(get_session_context));

    let attachments = Router::new()
        .route("/:attachment_id", delete(delete_attachment_upload))
        .route_layer(middleware::from_fn(get_session_context));

    let aprilfools2024 = Router::new()
        .route("/count", get(count_lootboxes))
        .route_layer(middleware::from_fn(get_session_context));

    Router::new()
        .nest("/auth", auth)
        .nest("/users", users)
        .nest("/hypesquad", hypesquad)
        .nest("/channels", channels)
        .nest("/gifs", gifs)
        .nest("/lootboxes", aprilfools2024)
        .nest("/safety-hub", safetyhub)
        .nest("/attachments", attachments)
        .route("/experiments", get(tracking::experiments))
        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
        .route("/metrics", post(tracking::science))
        .route_layer(middleware::from_fn(debug_body))
}

#[derive(Serialize)]
struct AccountStandingRes {
    account_standing: AccountStanding,
    classifications: Vec<Stub>,
    guild_classifications: Vec<Stub>
}

#[derive(Serialize)]
struct AccountStanding {
    /// 100 = All Good  
    /// 200 = Limited  
    /// 300 = Very Limited  
    /// 400 = At Risk  
    /// 500 = Suspended  
    state: i32
}

pub async fn account_standing() -> impl IntoResponse {
    Json(AccountStandingRes {
        account_standing: AccountStanding { state: 100 },
        classifications: vec![],
        guild_classifications: vec![],
    })
}