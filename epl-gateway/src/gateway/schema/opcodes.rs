use crate::gateway::dispatch::DispatchTypes;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::{debug, error};

use crate::gateway::schema::hello::Hello;
use crate::gateway::schema::presence::Presence;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::identify::Identify;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum OpCodes {
    #[default]
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum GatewayData {
    Dispatch {
        #[serde(flatten)]
        data: Box<DispatchTypes>,
    },
    Heartbeat(u64),
    Identify(Box<Identify>),
    PresenceUpdate(Box<Presence>),
    Hello(Box<Hello>),
}

pub fn get_opcode(msg: String) -> Result<(OpCodes, GatewayData), ()> {
    debug!("Decoding message: {}", &msg);

    // TODO: Figure out a less error-prone way of doing this, probably via custom deserialization
    let message_json: Result<GatewayMessage, serde_json::Error> = serde_json::from_str(&msg);

    if message_json.is_ok() {
        let output = message_json.unwrap();

        debug!("Decoded as Op: {:?}", &output.op);

        if output.d.is_none() {
            return Err(());
        }

        Ok((output.op, output.d.unwrap()))
    } else {
        error!("{:?}", message_json.err());
        Err(())
    }
}