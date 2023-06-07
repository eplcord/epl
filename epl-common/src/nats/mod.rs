use crate::RelationshipType;
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
    /// A friend request was sent to a user (sent to peer)
    RelationshipAdd {
        /// Creator of the request
        user_id: i64,
        /// Relationship type
        req_type: RelationshipType,
    },
    /// A friend request was ignored or a friend was removed (sent to peer)
    RelationshipRemove {
        /// Creator of the request
        user_id: i64,
        /// Relationship type
        req_type: RelationshipType,
    },
    /// A channel has been created
    ChannelCreate {
        /// ID of the channel created
        id: i64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Errors {
    /// The message that was sent was invalid
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
