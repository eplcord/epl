use axum::Router;

mod v1;
mod v3;
mod v6;
mod v9;

pub fn api() -> Router {
    Router::new()
        .nest("/v1", v1::routes::assemble_routes())
        .nest("/v3", v3::routes::assemble_routes())
        .nest("/v6", v6::routes::assemble_routes())
        .nest("/v9", v9::routes::assemble_routes())
}