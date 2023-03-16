use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Experiments {
    pub fingerprint: String,
    pub assignments: Vec<[i32; 5]>
}

pub async fn experiments() -> Json<Experiments> {
    Json(Experiments { fingerprint: "".to_string(), assignments: vec![] })
}

pub async fn science() -> &'static str {
    ""
}