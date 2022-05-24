use rocket::post;

#[post("/track")]
pub async fn track() -> &'static str {
    ""
}