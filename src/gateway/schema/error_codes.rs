use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ErrorCode {
    UnknownError,
    UnknownOpCode,
    DecodeError,
    NotAuthenticated,
    AuthenticationFailed,
    AlreadyAuthenticated,
    InvalidSeq,
    RateLimited,
    SessionTimedOut,
    InvalidShard,
    ShardingRequired,
    InvalidAPIVersion,
    InvalidIntents,
    DisallowedIntents
}

impl From<ErrorCode> for u16 {
    fn from(code: ErrorCode) -> u16 {
        match code {
            ErrorCode::UnknownError => 4000,
            ErrorCode::UnknownOpCode => 4001,
            ErrorCode::DecodeError => 4002,
            ErrorCode::NotAuthenticated => 4003,
            ErrorCode::AuthenticationFailed => 4004,
            ErrorCode::AlreadyAuthenticated => 4005,
            ErrorCode::InvalidSeq => 4007,
            ErrorCode::RateLimited => 4008,
            ErrorCode::SessionTimedOut => 4009,
            ErrorCode::InvalidShard => 4010,
            ErrorCode::ShardingRequired => 4011,
            ErrorCode::InvalidAPIVersion => 4012,
            ErrorCode::InvalidIntents => 4013,
            ErrorCode::DisallowedIntents => 4014
        }
    }
}

impl From<ErrorCode> for Cow<'static, str> {
    fn from(code: ErrorCode) -> Cow<'static, str> {
        Cow::from(match code {
            ErrorCode::UnknownError => "We're not sure what went wrong. Try reconnecting?",
            ErrorCode::UnknownOpCode => "You sent an invalid Gateway opcode or an invalid payload for an opcode. Don't do that!",
            ErrorCode::DecodeError => "You sent an invalid payload to us. Don't do that!",
            ErrorCode::NotAuthenticated => "You sent us a payload prior to identifying.",
            ErrorCode::AuthenticationFailed => "The account token sent with your identify payload is incorrect.",
            ErrorCode::AlreadyAuthenticated => "You sent more than one identify payload. Don't do that!",
            ErrorCode::InvalidSeq => "The sequence sent when resuming the session was invalid. Reconnect and start a new session.",
            ErrorCode::RateLimited => "Woah nelly! You're sending payloads to us too quickly. Slow it down! You will be disconnected on receiving this.",
            ErrorCode::SessionTimedOut => "Your session timed out. Reconnect and start a new one.",
            ErrorCode::InvalidShard => "You sent us an invalid shard when identifying.",
            ErrorCode::ShardingRequired => "The session would have handled too many guilds - you are required to shard your connection in order to connect.",
            ErrorCode::InvalidAPIVersion => "You sent an invalid version for the gateway.",
            ErrorCode::InvalidIntents => "You sent an invalid intent for a Gateway Intent. You may have incorrectly calculated the bitwise value.",
            ErrorCode::DisallowedIntents => "You sent a disallowed intent for a Gateway Intent. You may have tried to specify an intent that you have not enabled or are not approved for."
        })
    }
}

impl<'t> From<&'t ErrorCode> for u16 {
    fn from(code: &'t ErrorCode) -> u16 {
        (*code).into()
    }
}
