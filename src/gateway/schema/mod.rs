use serde::{Serialize, Deserialize};

use crate::gateway::schema::opcodes::{GatewayData, OpCodes};

pub(crate) mod opcodes;
pub(crate) mod identify;
pub(crate) mod presence;
pub(crate) mod error_codes;
pub(crate) mod ready;

#[derive(Deserialize, Serialize)]
pub struct GatewayMessage {
    pub op: OpCodes,
    pub d: GatewayData
}