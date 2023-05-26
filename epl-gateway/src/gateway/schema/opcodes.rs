use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::debug;

use crate::gateway::schema::hello::Hello;
use crate::gateway::schema::identify::Identify;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::presence::Presence;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum OpCodes {
    DISPATCH = 0,
    HEARTBEAT = 1,
    IDENTIFY = 2,
    PRESENCE_UPDATE = 3,
    VOICE_STATE_UPDATE = 4,
    HELLO = 10,
    HEARTBEAT_ACK = 11
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum GatewayData {
    DISPATCH {
        #[serde(flatten)]
        data: Box<DispatchData>,
    },
    HEARTBEAT(i32),
    IDENTIFY(Box<Identify>),
    PRESENCE_UPDATE(Box<Presence>),
    HELLO(Box<Hello>),
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum DispatchData {
    READY(super::ready::Ready),
    READY_SUPPLEMENTAL(super::ready::ReadySupplemental)
}

pub fn get_opcode(msg: String) -> Result<(OpCodes, GatewayData), ()> {
    debug!("Decoding message: {}", &msg);
    let message_json: Result<GatewayMessage, serde_json::Error> = serde_json::from_str(&msg);

    if let Ok(..) = message_json {
        let output = message_json.unwrap();

        debug!("Decoded as Op: {:?}", &output.op);

        Ok((output.op, output.d.unwrap()))
    } else {
        Err(())
    }
}
