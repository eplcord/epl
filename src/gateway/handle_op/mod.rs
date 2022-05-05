use log::debug;
use rocket::futures::SinkExt;
use rocket::futures::stream::SplitSink;
use warp::ws::{Message, WebSocket};

use crate::gateway::handle_op::handle_identify::handle_identify;
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::opcodes::{GatewayData, get_opcode, OpCodes};

mod handle_identify;

pub async fn handle_op(msg: Message, mut write: &mut SplitSink<WebSocket, Message>){
    let op = get_opcode(msg.clone());
    if op.is_ok() {
        let op = op.unwrap();

        match op.0 {
            OpCodes::DISPATCH => {
                debug!("DISPATCH");
                write.send(msg).await.expect("Failed to send message to gateway!");
            }
            OpCodes::HEARTBEAT => {
                debug!("HEARTBEAT");
                write.send(msg).await.expect("Failed to send message to gateway!");
            }
            OpCodes::IDENTIFY => {
                if let GatewayData::IDENTIFY(data) = op.1 {
                    handle_identify(data, &mut write).await;
                } else {
                    write.send(Message::close_with(ErrorCode::DecodeError, ErrorCode::DecodeError))
                        .await
                        .expect("Failed to close websocket!");
                }
            }
        }
    } else {
        write.send(Message::close_with(ErrorCode::UnknownOpCode, ErrorCode::UnknownOpCode))
            .await
            .expect("Failed to close websocket!");
    }
}