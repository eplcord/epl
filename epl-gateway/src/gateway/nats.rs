use epl_common::nats::Messages;
use crate::AppState;
use crate::gateway::dispatch::send_message;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::opcodes::OpCodes::INVALID_SESSION;
use crate::state::ThreadData;

pub async fn handle_nats_message(thread_data: &mut ThreadData, msg: Messages, state: &AppState) {
    match msg {
        Messages::InvalidateGatewaySession { session } => {
            if session.eq(thread_data.gateway_state.session_id.as_ref().unwrap()) {
                send_message(thread_data, GatewayMessage { op: INVALID_SESSION, ..Default::default()}).await;
            }
        },
        Messages::Error { .. } => {
            todo!();
        }
    }
}