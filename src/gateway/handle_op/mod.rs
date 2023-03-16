use tracing::debug;

use crate::gateway::handle_op::handle_identify::handle_identify;
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::opcodes::{GatewayData, get_opcode, OpCodes};

use axum::extract::ws::{Message, WebSocket, CloseFrame};

use futures::stream::SplitSink;

use futures::SinkExt;

mod handle_identify;

pub async fn handle_op(msg: String, mut write: &mut SplitSink<WebSocket, Message>){
    let op = get_opcode(msg.clone());
    if op.is_ok() {
        let op = op.unwrap();

        match op.0 {
            OpCodes::DISPATCH => {
                debug!("DISPATCH");
                write.send(axum::extract::ws::Message::Text(msg)).await.expect("Failed to send message to gateway!");
            }
            OpCodes::HEARTBEAT => {
                debug!("HEARTBEAT");
                write.send(axum::extract::ws::Message::Text(msg)).await.expect("Failed to send message to gateway!");
            }
            OpCodes::IDENTIFY => {
                if let GatewayData::IDENTIFY(data) = op.1 {
                    handle_identify(data, &mut write).await;
                } else {
                    write.send(Message::Close(Some(CloseFrame { code: ErrorCode::DecodeError.into(), reason: ErrorCode::DecodeError.into() })))
                        .await
                        .expect("Failed to close websocket!");
                }
            }
        }
    } else {
        write.send(Message::Close(Some(CloseFrame { code: ErrorCode::UnknownOpCode.into(), reason: ErrorCode::UnknownOpCode.into() })))
            .await
            .expect("Failed to close websocket!");
    }
}