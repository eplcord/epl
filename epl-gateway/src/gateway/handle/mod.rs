mod identify;

use tracing::debug;

use crate::gateway::handle::identify::handle_identify;
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::opcodes::{GatewayData, get_opcode, OpCodes};

use axum::extract::ws::{Message, CloseFrame};

use crate::AppState;
use crate::gateway::schema::GatewayMessage;
use crate::state::SOCKET;

pub async fn handle_op(msg: String, state: &AppState){
    let op = get_opcode(msg.clone());
    if op.is_ok() {
        let op = op.unwrap();

        match op.0 {
            OpCodes::HEARTBEAT => {
                let mut socket = SOCKET.get().lock().await;

                socket.as_mut().unwrap().inner.send(Message::Text(
                    serde_json::to_string(&GatewayMessage {
                        op: OpCodes::HEARTBEAT_ACK,
                        d: None,
                        s: None,
                        t: None,
                    })
                        .expect("Failed to serialize heartbeat ack!"),
                )
                ).await.expect("Failed to send ack to heartbeat!");
            }
            OpCodes::IDENTIFY => {
                if let GatewayData::IDENTIFY(data) = op.1 {
                    handle_identify(*data, state).await;
                } else {
                    let mut socket = SOCKET.get().lock().await;

                    socket.as_mut().unwrap().inner.send(Message::Close(Some(CloseFrame { code: ErrorCode::DecodeError.into(), reason: ErrorCode::DecodeError.into() })))
                        .await
                        .expect("Failed to close websocket!");
                }
            }
            _ => {
                debug!("Got an OP code that I don't have implemented!");
            }
        }
    } else {
        debug!("Got an OP code that I don't understand!");
    }
}