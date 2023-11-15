mod identify;

use tracing::debug;

use crate::gateway::handle::identify::handle_identify;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};

use crate::gateway::dispatch::{send_close, send_message};
use crate::gateway::schema::error_codes::ErrorCode::DecodeError;
use crate::gateway::schema::GatewayMessage;
use crate::state::ThreadData;
use crate::AppState;

pub async fn handle_op(thread_data: &mut ThreadData, op: Result<(OpCodes, GatewayData), ()>, state: &AppState) {
    if op.is_ok() {
        let op = op.unwrap();

        match op.0 {
            OpCodes::Heartbeat => {
                send_message(
                    thread_data,
                    GatewayMessage {
                        op: OpCodes::HeartbeatAck,
                        d: None,
                        s: None,
                        t: None,
                    },
                )
                .await;
            }
            OpCodes::Identify => {
                if let GatewayData::Identify(data) = op.1 {
                    handle_identify(thread_data, *data, state).await;
                } else {
                    send_close(thread_data, DecodeError).await;
                }
            }
            _ => {
                debug!("Got an OP code that I don't have implemented but I do understand!");
            }
        }
    } else {
        debug!("Got an OP code that I don't understand!");
    }
}
