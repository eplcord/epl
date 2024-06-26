use serde::{Deserialize, Serialize};

use crate::database::entities::{prelude::*, *};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::{Days, Utc};
use sea_orm::{ActiveValue, DatabaseConnection};

use crate::{gen_session_id, gen_token};
use zxcvbn::zxcvbn;

use sea_orm::*;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Clone)]
pub struct NewSessionError {
    pub kind: NewSessionEnum,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NewSessionEnum {
    SeaORM,
    BadUser,
}

/// Creates a new session for the specified user, returns the JWT
pub async fn generate_session(
    conn: &DatabaseConnection,
    user: i64,
) -> Result<String, NewSessionError> {
    let current_time = Utc::now().naive_utc();
    let expiry_time = current_time
        .checked_add_days(Days::new(30))
        .expect("Time has broken!");
    let token = gen_token();
    let session_id = gen_session_id();

    let new_session = session::ActiveModel {
        token: ActiveValue::Set(token.clone()),
        status: ActiveValue::Set(String::from("online")),
        os: ActiveValue::Set(None),
        platform: ActiveValue::Set(None),
        last_used: ActiveValue::Set(current_time),
        user_id: ActiveValue::Set(user),
        iat: ActiveValue::Set(current_time),
        exp: ActiveValue::Set(expiry_time),
        session_id: ActiveValue::Set(session_id),
        location: ActiveValue::Set(None),
    };

    Session::insert(new_session)
        .exec(conn)
        .await
        .map_err(|err| NewSessionError {
            kind: NewSessionEnum::SeaORM,
            message: err.to_string(),
        })?;

    Ok(token)
}

#[derive(Debug, Clone)]
pub struct GetSessionError {
    pub kind: GetSessionEnum,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum GetSessionEnum {
    SeaORM,
    BadUser,
}

pub async fn get_user_from_session_by_token(
    conn: &DatabaseConnection,
    token: &String,
) -> Result<user::Model, GetSessionError> {
    let session: Option<session::Model> = Session::find()
        .filter(session::Column::Token.eq(token))
        .one(conn)
        .await
        .expect("Failed to access db!");

    match session {
        None => Err(GetSessionError {
            kind: GetSessionEnum::BadUser,
            message: "Session not found!".to_string(),
        }),
        Some(session) => {
            let user: Option<user::Model> = User::find_by_id(session.user_id)
                .one(conn)
                .await
                .expect("Failed to access db!");

            match user {
                None => Err(GetSessionError {
                    kind: GetSessionEnum::BadUser,
                    message: "User not found!".to_string(),
                }),
                Some(user) => Ok(user),
            }
        }
    }
}

pub async fn get_session_by_token(
    conn: &DatabaseConnection,
    token: &String,
) -> Result<session::Model, GetSessionError> {
    let session: Option<session::Model> = Session::find()
        .filter(session::Column::Token.eq(token))
        .one(conn)
        .await
        .expect("Failed to access db!");

    match session {
        None => Err(GetSessionError {
            kind: GetSessionEnum::BadUser,
            message: "Session not found!".to_string(),
        }),
        Some(session) => Ok(session),
    }
}

pub async fn get_session_by_id(
    conn: &DatabaseConnection,
    id: &String,
) -> Result<session::Model, GetSessionError> {
    let session: Option<session::Model> = Session::find_by_id(id)
        .one(conn)
        .await
        .expect("Failed to access db!");

    match session {
        None => Err(GetSessionError {
            kind: GetSessionEnum::BadUser,
            message: "Session not found!".to_string(),
        }),
        Some(session) => Ok(session),
    }
}

pub async fn get_all_sessions(conn: &DatabaseConnection, id: &i64) -> Vec<session::Model> {
    Session::find()
        .filter(session::Column::UserId.eq(*id))
        .all(conn)
        .await
        .expect("Failed to access db!")
}

#[derive(Debug, Clone)]
pub struct NewUserError {
    pub kind: NewUserEnum,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NewUserEnum {
    SeaORM,
    BadUsername,
    BadPassword,
    WeakPassword,
    TooShortPassword,
    TooLongPassword,
    BadEmail,
}

impl From<argon2::password_hash::Error> for NewUserError {
    fn from(error: argon2::password_hash::Error) -> Self {
        NewUserError {
            kind: NewUserEnum::BadPassword,
            message: error.to_string(),
        }
    }
}

/// Creates a new user in the database, returns the ID of the user
pub async fn create_user(
    conn: &DatabaseConnection,
    data: user::ActiveModel,
) -> Result<i64, NewUserError> {
    User::insert(data.clone())
        .exec(conn)
        .await
        .map_err(|err| NewUserError {
            kind: NewUserEnum::SeaORM,
            message: err.to_string(),
        })?;

    UserSetting::insert(user_setting::ActiveModel {
        user: data.id.clone(),
        ..Default::default()
    }).exec(conn)
        .await
        .map_err(|err| NewUserError {
            kind: NewUserEnum::SeaORM,
            message: err.to_string(),
        })?;

    Frecency::insert(frecency::ActiveModel {
        user: data.id.clone(),
        ..Default::default()
    }).exec(conn)
        .await
        .map_err(|err| NewUserError {
            kind: NewUserEnum::SeaORM,
            message: err.to_string(),
        })?;

    Ok(data.id.unwrap())
}

/// Generates a password hash using the Argon2id algorithm
pub fn generate_password_hash(
    password: &str,
    other_fields: Vec<&str>,
) -> Result<String, NewUserError> {
    // Check if password is acceptable length
    if password.len() < 8 {
        return Err(NewUserError {
            kind: NewUserEnum::TooShortPassword,
            message: "Password is too short".to_string(),
        });
    } else if password.len() > 999 {
        return Err(NewUserError {
            kind: NewUserEnum::TooLongPassword,
            message: "Password is too long".to_string(),
        });
    }

    // Use zxcvbn to check password strength
    let zxcvbn_result = zxcvbn(password, other_fields.as_slice()).unwrap();

    if zxcvbn_result.score() < 2 {
        return Err(NewUserError {
            kind: NewUserEnum::WeakPassword,
            message: "Password is too weak".to_string(),
        });
    }

    let salt = SaltString::generate(&mut OsRng);
    let hasher = Argon2::default();

    let hash = hasher.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}
