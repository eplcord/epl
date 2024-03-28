use crate::AppState;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum::extract::Request;
use epl_common::database::auth::{get_session_by_token, get_user_from_session_by_token};
use epl_common::database::entities::{session, user};

#[derive(Clone)]
pub struct SessionContext {
    pub user: user::Model,
    pub session: session::Model,
}

pub async fn get_session_context(
    Extension(state): Extension<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = request
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .unwrap();

    return if let Ok(session) = get_session_by_token(&state.conn, &String::from(auth)).await {
        if let Ok(user) = get_user_from_session_by_token(&state.conn, &String::from(auth)).await {
            let context = SessionContext { user, session };

            request.extensions_mut().insert(context);

            Ok(next.run(request).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    };
}
