use axum::headers::authorization::{Bearer, Credentials};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::{Extension, TypedHeader};
use epl_common::database::auth::{get_session, get_user_from_session};
use epl_common::database::entities::{session, user};
use crate::AppState;

#[derive(Clone)]
pub struct SessionContext {
    pub user: user::Model,
    pub session: session::Model,
}

pub async fn get_session_context<B>(
    Extension(state): Extension<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth = request
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .unwrap();

    return if let Ok(session) = get_session(&state.conn, &String::from(auth)).await {
        if let Ok(user) = get_user_from_session(&state.conn, &String::from(auth)).await {
            let context = SessionContext {
                user,
                session,
            };

            request.extensions_mut().insert(context);

            Ok(next.run(request).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}