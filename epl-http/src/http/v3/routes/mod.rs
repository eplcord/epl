use axum::Router;

pub fn assemble_routes() -> Router {
    let auth = Router::new();

    Router::new()
        .nest("/auth", auth)
}