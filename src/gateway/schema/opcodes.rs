use log::debug;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use warp::ws::Message;

use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::identify::Identify;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum OpCodes {
    DISPATCH = 0,
    HEARTBEAT = 1,
    IDENTIFY = 2
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum GatewayData {
    DISPATCH,
    HEARTBEAT,
    IDENTIFY(Identify)
}

pub fn get_opcode(msg: Message) -> Result<(OpCodes, GatewayData), ()> {
    let msg = msg.to_str().unwrap();
    debug!("Decoding message: {}", &msg);
    let message_json: Result<GatewayMessage, serde_json::Error> = serde_json::from_str(msg);

    if message_json.is_ok() {
        let output = message_json.unwrap();

        debug!("Decoded as Op: {:?}", &output.op);

        Ok((output.op, output.d))
    } else {
        Err(())
    }
}