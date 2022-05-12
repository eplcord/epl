use std::time::SystemTime;

use diesel::Insertable;
use serde::{Deserialize, Serialize};

use crate::schema::{sessions, users};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use zxcvbn::zxcvbn;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    exp: usize,
    iat: usize
}

#[derive(Insertable, Clone)]
#[table_name="sessions"]
pub struct NewSession {
    pub uuid: uuid::Uuid,
    pub user_id: i64,
    pub iat: SystemTime,
    pub exp: SystemTime
}

#[derive(Debug, Clone)]
pub struct NewSessionError {
    pub kind: NewSessionEnum,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NewSessionEnum {
    Diesel,
    BadUser,
}

impl From<diesel::result::Error> for NewSessionError {
    fn from(error: diesel::result::Error) -> Self {
        NewSessionError {
            kind: NewSessionEnum::Diesel,
            message: error.to_string(),
        }
    }
}

/// Creates a new session for the specified user, returns the JWT
pub fn generate_session(conn: &diesel::PgConnection, user: i64) -> Result<String, NewSessionError> {
    use diesel::prelude::*;

    let current_time = SystemTime::now();
    let expiry_time = current_time + std::time::Duration::from_secs(60 * 60 * 24 * 30);
    let uuid = uuid::Uuid::new_v4();

    let new_session = NewSession {
        uuid: uuid.clone(),
        user_id: user.clone(),
        iat: current_time.clone(),
        exp: expiry_time.clone()
    };

    diesel::insert_into(sessions::table)
        .values(new_session.clone())
        .execute(&*conn)
        .map_err(|err| {
            return NewSessionError { kind: NewSessionEnum::Diesel, message: err.to_string() }
        })?;

    Ok(uuid.to_string())
}

#[derive(Insertable, Clone)]
#[table_name="users"]
pub struct NewUser {
    pub id: i64,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    pub password_hash: String,
    pub date_of_birth: chrono::NaiveDateTime,
    pub nsfw_allowed: bool,
}

#[derive(Debug, Clone)]
pub struct NewUserError {
    pub kind: NewUserEnum,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NewUserEnum {
    Diesel,
    BadUsername,
    BadPassword,
    WeakPassword,
    TooShortPassword,
    TooLongPassword,
    BadEmail,
}

impl From<diesel::result::Error> for NewUserError {
    fn from(error: diesel::result::Error) -> Self {
        NewUserError {
            kind: NewUserEnum::Diesel,
            message: error.to_string(),
        }
    }
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
pub fn create_user(conn: &diesel::PgConnection, data: NewUser) -> Result<i64, NewUserError> {
    use diesel::prelude::*;

    diesel::insert_into(users::table)
        .values(data.clone())
        .execute(&*conn)
        .map_err(|err| {
            return NewUserError { kind: NewUserEnum::Diesel, message: err.to_string() }
        })?;

    Ok(data.id)
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