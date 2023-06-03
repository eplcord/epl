use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::gateway::schema::opcodes::{GatewayData, OpCodes};

pub(crate) mod error_codes;
pub(crate) mod hello;
pub(crate) mod identify;
pub(crate) mod opcodes;
pub(crate) mod presence;
pub(crate) mod ready;
pub(crate) mod voice_state;

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct GatewayMessage {
    pub s: Option<i64>,
    pub t: Option<String>,
    pub op: OpCodes,
    pub d: Option<GatewayData>,
}
