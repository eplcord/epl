use serde::{Deserialize, Serialize};
use crate::gateway::schema::opcodes::OpCodes;
use crate::gateway::schema::ready::{Ready, ReadySupplemental};

pub(crate) mod ready;
pub(crate) mod ready_supplemental;

#[derive(Deserialize, Serialize)]
pub struct DispatchOp {
    /// OP code (0 for dispatch)
    op: OpCodes,
    /// Dispatch code
    t: DispatchTypes,
    /// Socket sequence number
    s: u64,
    /// Dispatch data
    d: DispatchData
}

#[derive(Deserialize, Serialize)]
pub enum DispatchTypes {
    READY,
    READY_SUPPLEMENTAL
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum DispatchData {
    READY(Ready),
    READY_SUPPLEMENTAL(ReadySupplemental)
}

pub async fn assemble_dispatch(t: DispatchTypes, d: DispatchData) -> DispatchOp {
    DispatchOp {
        op: OpCodes::DISPATCH,
        t,
        s: 0,
        d,
    }
}