use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Experiments {
    pub fingerprint: String,
    pub assignments: Vec<[i32; 8]>,
    pub guild_experiments: Vec<[i32; 10]>
}

pub async fn experiments() -> Json<Experiments> {
    Json(Experiments {
        fingerprint: "".to_string(),
        assignments: vec![],
        guild_experiments: vec![],
    })
}

pub async fn science() -> &'static str {
    ""
}
