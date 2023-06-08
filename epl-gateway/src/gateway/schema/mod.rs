use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::gateway::schema::opcodes::{GatewayData, OpCodes};

pub(crate) mod channels;
pub(crate) mod error_codes;
pub(crate) mod hello;
pub(crate) mod identify;
pub(crate) mod message;
pub(crate) mod opcodes;
pub(crate) mod presence;
pub(crate) mod ready;
pub(crate) mod relationships;
pub(crate) mod voice_state;

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct GatewayMessage {
    pub s: Option<i64>,
    pub t: Option<String>,
    pub op: OpCodes,
    pub d: Option<GatewayData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedUser {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
    pub id: String,
    pub public_flags: i64,
    pub username: String,
}
