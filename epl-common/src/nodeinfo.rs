use crate::Stub;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
pub struct NodeInfo {
    /// The schema version, must be 2.1.
    pub version: String,
    /// Metadata about server software in use.
    pub software: Software,
    /// The protocols supported on this server.
    pub protocols: Vec<String>,
    /// The third party sites this server can connect to via their application API.
    pub services: Services,
    #[serde(rename = "openRegistrations")]
    /// Whether this server allows open self-registration.
    pub open_registrations: bool,
    /// Usage statistics for this server.
    pub usage: Usage,
    /// Free form key value pairs for software specific values. Clients should not rely on any specific key present.
    pub metadata: LitecordMetadata,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
/// Metadata about server software in use.
pub struct Software {
    /// The canonical name of this server software.
    pub name: String,
    /// The version of this server software.
    pub version: String,
    /// The url of the source code repository of this server software.
    pub repository: Option<String>,
    /// The url of the homepage of this server software.
    pub homepage: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
/// The third party sites this server can connect to via their application API.
pub struct Services {
    /// The third party sites this server can retrieve messages from for combined display with regular traffic.
    pub inbound: Vec<String>,
    /// The third party sites this server can publish messages to on the behalf of a user.
    pub outbound: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
/// Usage statistics for this server.
pub struct Usage {
    /// Statistics about the users of this server.
    pub users: UsageUsers,
    #[serde(rename = "localPosts")]
    /// The amount of posts that were made by users that are registered on this server.
    pub local_posts: Option<u64>,
    #[serde(rename = "localComments")]
    /// The amount of comments that were made by users that are registered on this server.
    pub local_comments: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
/// Statistics about the users of this server.
pub struct UsageUsers {
    /// The total amount of on this server registered users.
    pub total: u64,
    #[serde(rename = "activeHalfyear")]
    /// The amount of users that signed in at least once in the last 180 days.
    pub active_half_year: Option<u64>,
    #[serde(rename = "activeMonth")]
    /// The amount of users that signed in at least once in the last 30 days.
    pub active_month: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[skip_serializing_none]
/// Litecord specific NodeInfo metadata
pub struct LitecordMetadata {
    /// Public name of the Litecord/Epl instance
    #[serde(rename = "nodeName")]
    pub node_name: String,
    /// Public description of the Litecord/Epl instance
    #[serde(rename = "nodeDescription")]
    pub node_description: String,
    /// If the Litecord/Epl instance should be classified as "private"
    pub private: bool,
    /// Features that this Litecord/Epl instance provides
    pub features: Vec<String>,
    /// Federation stub
    pub federation: Stub,
}
