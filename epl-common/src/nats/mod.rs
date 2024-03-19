use chrono::NaiveDateTime;
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
    /// A channel has been deleted
    ChannelDelete {
        /// ID of the channel deleted
        id: i64,
    },
    /// A message has been created
    MessageCreate {
        /// ID of the message
        id: i64,
    },
    /// A message has been updated
    MessageUpdate {
        /// ID of the message
        id: i64,
    },
    /// A message was deleted
    MessageDelete {
        /// ID of the message
        id: i64,
        /// ID of the channel the message was deleted in
        channel_id: i64,
        /// ID of the guild that the channel is in
        guild_id: Option<i64>,
    },
    /// A user has started typing in a channel
    TypingStarted {
        /// The channel this is occurring in
        channel_id: i64,
        /// The user that is typing
        user_id: i64,
        /// The timestamp the user started typing at
        timestamp: NaiveDateTime
    },
    /// A user has been added to a channel
    ChannelRecipientAdd {
        /// The channel this is occurring in
        channel_id: i64,
        /// The user that is being added
        user_id: i64,
    },
    /// A user has been removed from a channel
    ChannelRecipientRemove {
        /// The channel this is occurring in
        channel_id: i64,
        /// The user that is being removed
        user_id: i64,
    },
    /// A user has updated their notes about another user
    UserNoteUpdate {
        /// The creator of the note
        creator_id: i64,
        /// The subject of the note
        subject_id: i64
    }
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
