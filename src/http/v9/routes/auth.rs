use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Datelike;
use rand::Rng;
use rand::rngs::StdRng;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::database::entities::{prelude::*, *};

use crate::{AppState, EplOptions, Options, rustflake};
use crate::database::auth::{create_user, generate_password_hash, generate_session, NewUserEnum};
use crate::http::v9::errors::{APIErrorCode, APIErrorField, APIErrorMessage, throw_http_error};

pub async fn location_metadata() -> &'static str {
    ""
}

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

pub async fn register(
    Extension(state): Extension<AppState>,
    data: Json<RegisterRequest>
) -> impl IntoResponse {
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

        return Err((StatusCode::BAD_REQUEST, throw_http_error(APIErrorCode::InvalidFormBody, error).await))
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
    ).unwrap().and_hms_opt(0, 0, 0);

    // Check if the user is underage
    if date_of_birth.unwrap().year() >= (chrono::Local::now().year() - 13) {
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
        return Err((StatusCode::BAD_REQUEST, throw_http_error(APIErrorCode::InvalidFormBody, error).await));
    }

    let password_hash = password_hash.unwrap();

    let mut snowflake_factory =  rustflake::Snowflake::default();
    let mut rng: StdRng = rand::SeedableRng::from_entropy();

    let new_user_id = snowflake_factory.generate();
    let new_user_discriminator: i16 = rng.gen_range(0001..9999);

    // Check if NSFW channels should be allowed
    let nsfw_allowed = if date_of_birth.unwrap().year() >= (chrono::Local::now().year() - 18) {
        false
    } else {
        true
    };

    let new_user = user::ActiveModel {
        id: ActiveValue::Set(new_user_id.clone()),
        system: Default::default(),
        bot: Default::default(),
        username: ActiveValue::Set(data.0.username),
        discriminator: ActiveValue::Set(new_user_discriminator.to_string()),
        bio: Default::default(),
        pronouns: Default::default(),
        avatar: Default::default(),
        avatar_decoration: Default::default(),
        banner: Default::default(),
        email: ActiveValue::Set(data.0.email),
        phone: Default::default(),
        mfa_enabled: Default::default(),
        acct_verified: Default::default(),
        password_hash: ActiveValue::Set(password_hash),
        date_of_birth: ActiveValue::Set(date_of_birth),
        nsfw_allowed: ActiveValue::Set(nsfw_allowed),
        purchased_flags: Default::default(),
        premium_flags: Default::default(),
        premium_type: Default::default(),
        banner_colour: Default::default(),
        flags: Default::default(),
        premium_since: Default::default(),
    };

    let user =  create_user(&state.conn, new_user).await;

    let user_id = match user {
        Ok(user) => user,
        Err(_) => {
            error.push(APIErrorField::Email {
                _errors: vec![
                    APIErrorMessage {
                        code: "SERVER_ERROR".to_string(),
                        message: "A server error occurred, try again later.".to_string()
                    }
                ]
            });

            return Err((StatusCode::BAD_REQUEST, throw_http_error(APIErrorCode::InvalidFormBody, error).await));
        }
    };

    let token = generate_session(&state.conn, user_id.clone()).await.unwrap();

    info!("New account registered: {}", user_id);

    Ok(Json(RegisterResponse{ token }))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub login: String,
    pub password: String
}

#[derive(Serialize)]
pub struct LoginRes {
    pub token: String
}

pub async fn login(
    Extension(state): Extension<AppState>,
    data: Json<LoginReq>
) -> impl IntoResponse {
    let mut error = Vec::new();

    // Check if user exists
    let requested_user: Option<user::Model> = User::find()
        .filter(user::Column::Email.eq(&data.login))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    let requested_user = match requested_user {
        None => {
            error.push(APIErrorField::Email {
                _errors: vec![
                    APIErrorMessage {
                        code: "INVALID_LOGIN".to_string(),
                        message: "Login or password is invalid.".to_string()
                    }
                ]
            });

            error.push(APIErrorField::Password {
                _errors: vec![
                    APIErrorMessage {
                        code: "INVALID_LOGIN".to_string(),
                        message: "Login or password is invalid.".to_string()
                    }
                ]
            });

            return Err((StatusCode::BAD_REQUEST, throw_http_error(APIErrorCode::InvalidFormBody, error).await));
        }
        Some(user) => {
            user
        }
    };

    // Verify password
    let password_hash = PasswordHash::new(&requested_user.password_hash).expect("Failed to parse password hash!");

    if Argon2::default().verify_password(data.password.as_bytes(), &password_hash).is_err() {
        error.push(APIErrorField::Email {
            _errors: vec![
                APIErrorMessage {
                    code: "INVALID_LOGIN".to_string(),
                    message: "Login or password is invalid.".to_string()
                }
            ]
        });

        error.push(APIErrorField::Password {
            _errors: vec![
                APIErrorMessage {
                    code: "INVALID_LOGIN".to_string(),
                    message: "Login or password is invalid.".to_string()
                }
            ]
        });

        return Err((StatusCode::BAD_REQUEST, throw_http_error(APIErrorCode::InvalidFormBody, error).await));
    }

    // Generate session
    let token = generate_session(&state.conn, requested_user.id.clone()).await.unwrap();

    Ok(Json(LoginRes { token }))
}