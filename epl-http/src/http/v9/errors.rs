use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub async fn throw_http_error(
    error: APIErrorCode,
    error_messages: Vec<APIErrorField>,
) -> Json<APIError> {
    // I really don't like doing this, maybe play around with serde_as more
    let mut error_messages_vec = Vec::<(String, APIErrorField)>::new();

    for i in error_messages {
        error_messages_vec.push((String::from(i.clone()), i));
    }

    Json(APIError {
        code: u32::from(error),
        message: String::from(error),
        errors: error_messages_vec,
    })
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct APIError {
    code: u32,
    message: String,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    errors: Vec<(String, APIErrorField)>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
#[serde(untagged)]
pub enum APIErrorField {
    #[serde(alias = "password")]
    Password { _errors: Vec<APIErrorMessage> },
    #[serde(alias = "username")]
    Username { _errors: Vec<APIErrorMessage> },
    #[serde(alias = "login")]
    Login { _errors: Vec<APIErrorMessage> },
    #[serde(alias = "email")]
    Email { _errors: Vec<APIErrorMessage> },
    #[serde(alias = "date_of_birth")]
    DateOfBirth { _errors: Vec<APIErrorMessage> },
}

impl From<APIErrorField> for String {
    fn from(field: APIErrorField) -> String {
        match field {
            APIErrorField::Password { .. } => "password",
            APIErrorField::Username { .. } => "username",
            APIErrorField::Login { .. } => "login",
            APIErrorField::Email { .. } => "email",
            APIErrorField::DateOfBirth { .. } => "date_of_birth",
        }
        .parse()
        .unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct APIErrorMessage {
    pub(crate) code: String,
    pub(crate) message: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum APIErrorCode {
    UnknownAccount,
    UnknownUser,
    DisabledAccount, 
    Unauthorized,
    PasswordDoesNotMatch,
    InvalidFormBody,
    FriendRequestBlocked,
    CannotSendFriendRequestToSelf,
    UnknownEmoji
}

impl From<APIErrorCode> for u32 {
    fn from(code: APIErrorCode) -> u32 {
        match code {
            APIErrorCode::UnknownAccount => 10001,
            APIErrorCode::UnknownUser => 10013,
            APIErrorCode::DisabledAccount => 20013,
            APIErrorCode::Unauthorized => 40001,
            APIErrorCode::PasswordDoesNotMatch => 50018,
            APIErrorCode::InvalidFormBody => 50035,
            APIErrorCode::CannotSendFriendRequestToSelf => 80003,
            APIErrorCode::FriendRequestBlocked => 80001,
            APIErrorCode::UnknownEmoji => 10014
        }
    }
}

impl From<APIErrorCode> for String {
    fn from(code: APIErrorCode) -> String {
        match code {
            APIErrorCode::UnknownAccount => "Unknown Account".to_string(),
            APIErrorCode::UnknownUser => "Unknown User".to_string(),
            APIErrorCode::DisabledAccount => "This account is disabled.".to_string(),
            APIErrorCode::Unauthorized => "Unauthorized".to_string(),
            APIErrorCode::PasswordDoesNotMatch => "Password does not match".to_string(),
            APIErrorCode::InvalidFormBody => "Unknown Form Body".to_string(),
            APIErrorCode::CannotSendFriendRequestToSelf => {
                "Cannot send friend request to self".to_string()
            }
            APIErrorCode::FriendRequestBlocked => "Friend request blocked".to_string(),
            APIErrorCode::UnknownEmoji => "Unknown Emoji".to_string()
        }
    }
}

impl From<APIErrorCode> for StatusCode {
    fn from(code: APIErrorCode) -> StatusCode {
        match code {
            APIErrorCode::UnknownAccount => StatusCode::BAD_REQUEST,
            APIErrorCode::UnknownUser => StatusCode::NOT_FOUND,
            APIErrorCode::DisabledAccount => StatusCode::BAD_REQUEST,
            APIErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            APIErrorCode::PasswordDoesNotMatch => StatusCode::BAD_REQUEST,
            APIErrorCode::InvalidFormBody => StatusCode::BAD_REQUEST,
            APIErrorCode::CannotSendFriendRequestToSelf => StatusCode::BAD_REQUEST,
            APIErrorCode::FriendRequestBlocked => StatusCode::BAD_REQUEST,
            APIErrorCode::UnknownEmoji => StatusCode::BAD_REQUEST
        }
    }
}

impl<'t> From<&'t APIErrorCode> for u32 {
    fn from(code: &'t APIErrorCode) -> u32 {
        (*code).into()
    }
}
