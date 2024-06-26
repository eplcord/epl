use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::Datelike;
use epl_common::options::{EplOptions, Options};
use rand::rngs::StdRng;
use rand::Rng;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use epl_common::database::entities::{prelude::*, *};

use crate::authorization_extractor::SessionContext;
use crate::http::v9::errors::{throw_http_error, APIErrorCode, APIErrorField, APIErrorMessage};
use epl_common::nats::send_nats_message;
use crate::AppState;
use epl_common::database::auth::{
    create_user, generate_password_hash, generate_session, get_all_sessions, get_session_by_id,
    NewUserEnum,
};
use epl_common::flags::{get_user_flags, UserFlags};
use epl_common::nats::Messages;
use epl_common::rustflake;

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
    invite: Option<String>,
    global_name: Option<String>
}

#[derive(Serialize)]
pub struct RegisterResponse {
    token: String,
}

pub async fn register(
    Extension(state): Extension<AppState>,
    data: Json<RegisterRequest>,
) -> impl IntoResponse {
    let options = EplOptions::get();
    let mut error = Vec::new();

    // Exit early if registration is disabled
    if !options.registration {
        error.push(APIErrorField::Email {
            _errors: vec![APIErrorMessage {
                code: "REGISTRATION_DISABLED".to_string(),
                message: "Registration has been disabled".to_string(),
            }],
        });

        return Err((
            StatusCode::BAD_REQUEST,
            throw_http_error(APIErrorCode::InvalidFormBody, error).await,
        ));
    }

    let password_hash = generate_password_hash(
        &data.password,
        vec![data.0.username.as_str(), data.0.email.as_str()],
    );

    if password_hash.is_err() {
        match password_hash.clone().err().unwrap().kind {
            NewUserEnum::BadPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![APIErrorMessage {
                        code: "INVALID_PASSWORD".to_string(),
                        message: "Password is invalid".to_string(),
                    }],
                });
            }
            NewUserEnum::TooShortPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![APIErrorMessage {
                        code: "INVALID_PASSWORD".to_string(),
                        message: "Password must be over 8 characters long".to_string(),
                    }],
                });
            }
            NewUserEnum::TooLongPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![APIErrorMessage {
                        code: "INVALID_PASSWORD".to_string(),
                        message: "Password must be under 999 characters long".to_string(),
                    }],
                });
            }
            NewUserEnum::WeakPassword => {
                error.push(APIErrorField::Password {
                    _errors: vec![APIErrorMessage {
                        code: "INVALID_PASSWORD".to_string(),
                        message: "Password is too weak or common to use".to_string(),
                    }],
                });
            }
            _ => {}
        }
    }

    let date_of_birth = chrono::NaiveDate::parse_from_str(&data.0.date_of_birth, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(0, 0, 0);

    // Check if the user is underage
    if date_of_birth.unwrap().year() >= (chrono::Local::now().year() - 13) {
        error.push(APIErrorField::DateOfBirth {
            _errors: vec![APIErrorMessage {
                code: "DATE_OF_BIRTH_UNDERAGE".to_string(),
                message: "You need to be 13 or older in order to use Discord.".to_string(),
            }],
        });
    }

    if !error.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            throw_http_error(APIErrorCode::InvalidFormBody, error).await,
        ));
    }

    let password_hash = password_hash.unwrap();

    let mut snowflake_factory = rustflake::Snowflake::default();
    let mut rng: StdRng = rand::SeedableRng::from_entropy();

    let new_user_id = snowflake_factory.generate();
    let new_user_discriminator: i16 = if options.pomelo {
        0
    } else {
        rng.gen_range(1..9999)
    };

    let display_name = if data.0.global_name.clone().is_some_and(|x| !x.is_empty()) {
        data.0.global_name
    } else {
        None
    };

    // Check if NSFW channels should be allowed
    let nsfw_allowed = date_of_birth.unwrap().year() < (chrono::Local::now().year() - 18);

    let new_user = user::ActiveModel {
        id: ActiveValue::Set(new_user_id),
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
        accent_color: Default::default(),
        display_name: Set(display_name),
        legacy_name: Default::default(),
    };

    let user = create_user(&state.conn, new_user).await;

    let user_id = match user {
        Ok(user) => user,
        Err(_) => {
            error.push(APIErrorField::Email {
                _errors: vec![APIErrorMessage {
                    code: "SERVER_ERROR".to_string(),
                    message: "A server error occurred, try again later.".to_string(),
                }],
            });

            return Err((
                StatusCode::BAD_REQUEST,
                throw_http_error(APIErrorCode::InvalidFormBody, error).await,
            ));
        }
    };

    let token = generate_session(&state.conn, user_id).await.unwrap();

    info!("New account registered: {}", user_id);

    Ok(Json(RegisterResponse { token }))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub login: String,
    pub password: String,
    pub undelete: bool,
}

#[derive(Serialize)]
pub struct LoginRes {
    pub token: String,
}

pub async fn login(
    Extension(state): Extension<AppState>,
    data: Json<LoginReq>,
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
                _errors: vec![APIErrorMessage {
                    code: "INVALID_LOGIN".to_string(),
                    message: "Login or password is invalid.".to_string(),
                }],
            });

            error.push(APIErrorField::Password {
                _errors: vec![APIErrorMessage {
                    code: "INVALID_LOGIN".to_string(),
                    message: "Login or password is invalid.".to_string(),
                }],
            });

            return Err((
                StatusCode::BAD_REQUEST,
                throw_http_error(APIErrorCode::InvalidFormBody, error).await,
            ));
        }
        Some(user) => user,
    };

    // Verify password
    let password_hash =
        PasswordHash::new(&requested_user.password_hash).expect("Failed to parse password hash!");

    if Argon2::default()
        .verify_password(data.password.as_bytes(), &password_hash)
        .is_err()
    {
        error.push(APIErrorField::Email {
            _errors: vec![APIErrorMessage {
                code: "INVALID_LOGIN".to_string(),
                message: "Login or password is invalid.".to_string(),
            }],
        });

        error.push(APIErrorField::Password {
            _errors: vec![APIErrorMessage {
                code: "INVALID_LOGIN".to_string(),
                message: "Login or password is invalid.".to_string(),
            }],
        });

        return Err((
            StatusCode::BAD_REQUEST,
            throw_http_error(APIErrorCode::InvalidFormBody, error).await,
        ));
    }

    // Check if user has the disabled bitflag
    if get_user_flags(requested_user.flags).contains(&UserFlags::Disabled) {
        if data.undelete {
            let mut user = requested_user.clone().into_active_model();
            
            user.flags = Set(requested_user.flags - UserFlags::Disabled as i64);
            
            user.update(&state.conn).await.expect("Failed to update user!");
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                throw_http_error(APIErrorCode::DisabledAccount, vec![]).await,
            )); 
        }
    }

    // Generate session
    let token = generate_session(&state.conn, requested_user.id)
        .await
        .unwrap();

    Ok(Json(LoginRes { token }))
}

// TODO: Research more into what these mean
#[derive(Deserialize)]
pub struct LogoutReq {
    pub provider: Option<String>,
    pub voip_provider: Option<String>,
}

pub async fn logout(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    _data: Json<LogoutReq>,
) -> impl IntoResponse {
    match session_context.session.delete(&state.conn).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn verify_email(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    // Stub this and automatically verify the user
    // TODO: Once we have SMTP, queue a verification email to be sent

    if get_user_flags(session_context.user.flags).contains(&UserFlags::VerifiedEmail) {
        return StatusCode::BAD_REQUEST;
    }

    let mut updated_user: user::ActiveModel = session_context.user.into_active_model();

    updated_user.acct_verified = Set(true);
    updated_user.flags = Set(updated_user.flags.unwrap() + UserFlags::VerifiedEmail as i64);

    match updated_user.update(&state.conn).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Serialize)]
pub struct SessionsRes {
    pub user_sessions: Vec<Session>,
}

#[derive(Serialize)]
pub struct Session {
    id_hash: String,
    approx_last_used_time: String,
    client_info: ClientInfo,
}

#[derive(Serialize)]
pub struct ClientInfo {
    os: String,
    platform: String,
    location: String,
}

pub async fn sessions(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let sessions: Vec<Session> = get_all_sessions(&state.conn, &session_context.user.id)
        .await
        .into_iter()
        .map(|session| Session {
            id_hash: session.session_id,
            approx_last_used_time: session.last_used.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            client_info: ClientInfo {
                os: session.os.unwrap_or(String::from("Unknown")),
                platform: session.platform.unwrap_or(String::from("Unknown")),
                location: session.location.unwrap_or(String::from("Unknown")),
            },
        })
        .collect();

    Json(SessionsRes {
        user_sessions: sessions,
    })
}

#[derive(Deserialize)]
pub struct LogoutSessionReq {
    password: String,
    session_id_hashes: Vec<String>,
}

pub async fn logout_session(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    data: Json<LogoutSessionReq>,
) -> impl IntoResponse {
    // Verify password
    let password_hash = PasswordHash::new(&session_context.user.password_hash)
        .expect("Failed to parse password hash!");

    if Argon2::default()
        .verify_password(data.password.as_bytes(), &password_hash)
        .is_err()
    {
        return (
            StatusCode::from(APIErrorCode::PasswordDoesNotMatch),
            throw_http_error(APIErrorCode::PasswordDoesNotMatch, vec![]).await,
        )
            .into_response();
    }

    for i in &data.session_id_hashes {
        let session = get_session_by_id(&state.conn, i).await;

        match session {
            Ok(session) => {
                session
                    .into_active_model()
                    .delete(&state.conn)
                    .await
                    .expect("Failed to remove session from db!");

                send_nats_message(
                    &state.nats_client,
                    session_context.user.id.to_string(),
                    Messages::InvalidateGatewaySession { session: i.clone() },
                )
                .await;
            }
            Err(_) => {
                // TODO: see what discord returns for an invalid session id
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        }
    }

    (StatusCode::OK).into_response()
}
