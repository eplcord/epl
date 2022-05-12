use chrono::Datelike;
use log::debug;
use rand::Rng;
use rand::rngs::StdRng;
use rocket::{get, post};
use rocket::response::status::BadRequest;
use rocket::serde::{Deserialize, json::Json, Serialize};

use crate::{EplOptions, Options, rustflake};
use crate::database::auth::{create_user, generate_password_hash, generate_session, NewUser, NewUserEnum};
use crate::database::EplDb;
use crate::http::routes::v9::http_errors::{APIError, APIErrorCode, APIErrorField, APIErrorMessage, throw_http_error};

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
    date_of_birth: String,
    consent: bool,
    captcha_key: Option<String>,
    gift_code_sku_id: Option<String>,
    invite: Option<String>
}

#[derive(Serialize)]
pub struct RegisterResponse {
    token: String
}

#[get("/location-metadata")]
pub async fn location_metadata() -> &'static str {
    ""
}

#[post("/register", data = "<data>")]
pub async fn register(conn: EplDb, data: Json<RegisterRequest>) -> Result<Json<RegisterResponse>, BadRequest<Json<APIError>>> {
    let options = EplOptions::get();
    let mut error = Vec::new();

    // Exit early if registration is disabled
    if !options.registration {
        error.push(APIErrorField::Email {
            _errors: vec![
                APIErrorMessage {
                    code: "REGISTRATION_DISABLED".to_string(),
                    message: "Registration has been disabled".to_string()
                }
            ]
        });

        return Err(
            BadRequest(
                Some(
                    throw_http_error(APIErrorCode::InvalidFormBody, error).await
                )))
    }

    let password_hash = generate_password_hash(&data.password,
                                               vec![data.0.username.as_str(), data.0.email.as_str()]);

    if password_hash.is_err() {
        match password_hash.clone().err().unwrap().kind {
            NewUserEnum::BadPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![
                        APIErrorMessage {
                            code: "INVALID_PASSWORD".to_string(),
                            message: "Password is invalid".to_string()
                        }
                    ]
                });
            }
            NewUserEnum::TooShortPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![
                        APIErrorMessage {
                            code: "INVALID_PASSWORD".to_string(),
                            message: "Password must be over 8 characters long".to_string()
                        }
                    ]
                });
            }
            NewUserEnum::TooLongPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![
                        APIErrorMessage {
                            code: "INVALID_PASSWORD".to_string(),
                            message: "Password must be under 999 characters long".to_string()
                        }
                    ]
                });
            }
            NewUserEnum::WeakPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![
                        APIErrorMessage {
                            code: "INVALID_PASSWORD".to_string(),
                            message: "Password is too weak or common to use".to_string()
                        }
                    ]
                });
            }
            _ => {}
        }
    }

    let date_of_birth = chrono::NaiveDate::parse_from_str(
        &data.0.date_of_birth, "%Y-%m-%d"
    ).unwrap().and_hms(0, 0, 0);

    // Check if the user is underage
    if date_of_birth.year() >= (chrono::Local::now().year() - 13) {
        error.push(APIErrorField::DateOfBirth {
            _errors: vec![
                APIErrorMessage {
                    code: "DATE_OF_BIRTH_UNDERAGE".to_string(),
                    message: "You need to be 13 or older in order to use Discord.".to_string()
                }
            ]
        });
    }

    if !error.is_empty() {
        return Err(
            BadRequest(
                Some(
                    throw_http_error(APIErrorCode::InvalidFormBody, error).await
                )))
    }

    let password_hash = password_hash.unwrap();

    let mut snowflake_factory =  rustflake::Snowflake::default();
    let mut rng: StdRng = rand::SeedableRng::from_entropy();

    let new_user_id = snowflake_factory.generate();
    let new_user_discriminator: i16 = rng.gen_range(0001..9999);

    // Check if NSFW channels should be allowed
    let nsfw_allowed = if date_of_birth.year() >= (chrono::Local::now().year() - 18) {
        false
    } else {
        true
    };

    let new_user = NewUser {
        id: new_user_id.clone(),
        username: data.0.username,
        discriminator: new_user_discriminator.to_string(),
        email: data.0.email,
        password_hash,
        date_of_birth,
        nsfw_allowed
    };

    let user_id = conn.run(move |c| {
        create_user(c, new_user)
    }).await;

    let user_id = user_id.unwrap();

    let token = conn.run(move |c| {
        generate_session(c, user_id.clone())
    }).await;

    let token = token.unwrap();

    debug!("New account registered: {}", user_id);

    Ok(Json(RegisterResponse{ token }))
}