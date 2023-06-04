use epl_common::nats::Messages;
use crate::AppState;
use crate::gateway::dispatch::relationships::{dispatch_relationship_add, dispatch_relationship_remove};
use crate::gateway::dispatch::send_message;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::opcodes::OpCodes::InvalidSession;
use crate::state::ThreadData;

pub async fn handle_nats_message(thread_data: &mut ThreadData, msg: Messages, state: &AppState) {
    match msg {
        Messages::InvalidateGatewaySession { session } => {
            if session.eq(thread_data.gateway_state.session_id.as_ref().unwrap()) {
                send_message(thread_data, GatewayMessage { op: InvalidSession, ..Default::default()}).await;
            }
        },
        Messages::Error { .. } => {
            todo!();
        }
        Messages::RelationshipAdd { user_id, req_type } => {
            dispatch_relationship_add(thread_data, state, user_id, req_type).await;
        }
        Messages::RelationshipRemove { user_id, req_type } => {
            dispatch_relationship_remove(thread_data, state, user_id, req_type).await;
        }
    }
}