use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum Messages {
    /// Invalidates a running gateway session and causes a user to be logged out
    InvalidateGatewaySession {
        /// ID of the session that should be invalidated
        session: String,
    },
    /// An error in the NATS protocol
    Error {
        /// Error enum entry
        error: Errors,
        /// Human readable error message
        message: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Errors {
    InvalidMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NatsKV {
    pub user_id: Option<i64>,
    pub session_id: Option<String>,
    pub bot: Option<bool>,
    pub large_threshold: Option<i8>,
    pub current_shard: Option<i8>,
    pub shard_count: Option<i8>,
    pub intents: Option<i8>,
}