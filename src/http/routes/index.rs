use rocket::get;
use rocket_dyn_templates::Template;
use crate::{EplOptions, Options, VERSION};
use serde::Serialize;

#[derive(Serialize)]
pub struct IndexTemplate {
    instance_name: String,
    version: String,
}

#[get("/")]
pub async fn index() -> Template {
    let options = EplOptions::get();

    let context = IndexTemplate{
        instance_name: options.name.to_string(),
        version: VERSION.to_string()
    };

    Template::render("index", context)
}