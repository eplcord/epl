use axum::{
    body::Body,
    extract:: Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use tracing::debug;

pub async fn debug_body(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    // Only log body when built in debug
    if cfg!(debug_assertions) {
        let (parts, body) = request.into_parts();

        let bytes = body
            .collect()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
            .to_bytes();

        debug!(body = ?bytes);

        Ok(next.run(Request::from_parts(parts, Body::from(bytes))).await)
    } else {
        Ok(next.run(request).await)
    }
}