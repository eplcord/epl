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
pub(crate) mod reactions;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct GatewayMessage {
    pub s: Option<i64>,
    pub t: Option<String>,
    pub op: OpCodes,
    pub d: Option<GatewayData>,
}