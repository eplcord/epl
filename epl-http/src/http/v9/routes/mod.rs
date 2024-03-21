mod auth;
mod channels;
mod hypesquad;
mod tracking;
mod users;

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
use axum::{middleware, Router};
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
        .route("/relationships/:id", delete(delete_relationship))
        .route("/relationships/:id", put(modify_relationship))
        .route("/channels", post(new_dm_channel))
        .route("/disable", post(disable_account))
        .route("/profile", patch(update_profile))
        .route("/notes/:id", get(get_notes))
        .route("/notes/:id", put(put_notes))
        .route("/pomelo", post(pomelo))
        .route("/", patch(update_user));

    let users = Router::new()
        .nest("/@me", atme)
        .route("/:id/profile", get(profile))
        // Workaround
        .route("/%40me/profile", patch(update_profile))
        .route_layer(middleware::from_fn(get_session_context));

    let hypesquad = Router::new()
        .route("/online", post(join_hypesquad))
        .route("/online", delete(leave_hypesquad))
        .route_layer(middleware::from_fn(get_session_context));

    let channels = Router::new()
        .route("/:id/messages/:id", patch(edit_message))
        .route("/:id/messages/:id", delete(delete_message))
        .route("/:id/messages", get(get_messages))
        .route("/:id/messages", post(send_message))
        .route("/:id/typing", post(typing))
        .route("/:id/recipients/:id", put(add_user_to_channel))
        .route("/:id/recipients/:id", delete(remove_user_from_channel))
        .route("/:id", patch(modify_channel))
        .route_layer(middleware::from_fn(get_session_context));

    Router::new()
        .nest("/auth", auth)
        .nest("/users", users)
        .nest("/hypesquad", hypesquad)
        .nest("/channels", channels)
        .route("/experiments", get(tracking::experiments))
        .route("/science", post(tracking::science))
        .route("/track", post(tracking::science))
}
