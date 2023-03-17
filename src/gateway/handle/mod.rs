mod identify;

use tracing::debug;

use crate::gateway::handle::identify::handle_identify;
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::opcodes::{GatewayData, get_opcode, OpCodes};

use axum::extract::ws::{Message, CloseFrame};

use crate::AppState;
use crate::state::SOCKET;

pub async fn handle_op(msg: String, state: &AppState){
    let op = get_opcode(msg.clone());
    if op.is_ok() {
        let op = op.unwrap();

        match op.0 {
            OpCodes::DISPATCH => {
                debug!("DISPATCH");
                let mut socket = SOCKET.get().lock().await;

                socket.inner.send(axum::extract::ws::Message::Text(msg)).await.expect("Failed to send message to gateway!");
            }
            OpCodes::HEARTBEAT => {
                debug!("HEARTBEAT");
                let mut socket = SOCKET.get().lock().await;

                socket.inner.send(axum::extract::ws::Message::Text(msg)).await.expect("Failed to send message to gateway!");
            }
            OpCodes::IDENTIFY => {
                if let GatewayData::IDENTIFY(data) = op.1 {
                    handle_identify(*data, state).await;
                } else {
                    let mut socket = SOCKET.get().lock().await;

                    socket.inner.send(Message::Close(Some(CloseFrame { code: ErrorCode::DecodeError.into(), reason: ErrorCode::DecodeError.into() })))
                        .await
                        .expect("Failed to close websocket!");
                }
            }
        }
    } else {
        let mut socket = SOCKET.get().lock().await;

        socket.inner.send(Message::Close(Some(CloseFrame { code: ErrorCode::UnknownOpCode.into(), reason: ErrorCode::UnknownOpCode.into() })))
            .await
            .expect("Failed to close websocket!");
    }
}