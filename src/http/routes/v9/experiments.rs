use rocket::{options, post};
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Experiments {
    pub fingerprint: String,
    pub assignments: Vec<[i32; 5]>
}

#[options("/experiments")]
pub async fn experiments() -> Json<Experiments> {
    Json(Experiments{ fingerprint: "".to_string(), assignments: vec![] })
}

#[post("/science")]
pub async fn science() -> &'static str {
    ""
}