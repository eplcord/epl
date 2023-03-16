use serde::{Deserialize, Serialize};

use crate::database::entities::{prelude::*, *};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use chrono::{Days, Utc};
use sea_orm::{ActiveValue, DatabaseConnection};

use zxcvbn::zxcvbn;
use crate::util::gen_token;

use sea_orm::*;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    exp: usize,
    iat: usize
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
pub async fn generate_session(conn: &DatabaseConnection, user: i64) -> Result<String, NewSessionError> {
    let current_time = Utc::now().naive_utc();
    let expiry_time = current_time.checked_add_days(Days::new(30)).expect("Time has broken!");
    let token = gen_token();

    let new_session = session::ActiveModel {
        token: ActiveValue::Set(token.clone()),
        user_id: ActiveValue::Set(user),
        iat: ActiveValue::Set(current_time),
        exp: ActiveValue::Set(expiry_time)
    };

    Session::insert(new_session)
        .exec(conn)
        .await
        .map_err(|err| {
            return NewSessionError { kind: NewSessionEnum::SeaORM, message: err.to_string() }
        })?;

    Ok(token)
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
pub async fn create_user(conn: &DatabaseConnection, data: user::ActiveModel) -> Result<i64, NewUserError> {
    User::insert(data.clone())
        .exec(conn)
        .await
        .map_err(|err| {
            return NewUserError { kind: NewUserEnum::SeaORM, message: err.to_string() }
        })?;

    Ok(data.id.unwrap())
}

/// Generates a password hash using the Argon2id algorithm
pub fn generate_password_hash(password: &str, other_fields: Vec<&str>) -> Result<String, NewUserError> {
    // Check if password is acceptable length
    if password.len() < 8 {
        return Err(NewUserError { kind: NewUserEnum::TooShortPassword, message: "Password is too short".to_string() });
    } else if password.len() > 999 {
        return Err(NewUserError { kind: NewUserEnum::TooLongPassword, message: "Password is too long".to_string() });
    }

    // Use zxcvbn to check password strength
    let zxcvbn_result = zxcvbn(password, other_fields.as_slice()).unwrap();

    if zxcvbn_result.score() < 2 {
        return Err(NewUserError { kind: NewUserEnum::WeakPassword, message: "Password is too weak".to_string() });
    }

    let salt = SaltString::generate(&mut OsRng);
    let hasher = Argon2::default();

    let hash = hasher.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}