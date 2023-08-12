use crate::gateway::dispatch::channel::dispatch_channel_create;
use crate::gateway::dispatch::message::{dispatch_message, dispatch_message_delete, DispatchMessageTypes};
use crate::gateway::dispatch::relationships::{
    dispatch_relationship_add, dispatch_relationship_remove,
};
use crate::gateway::dispatch::send_message;
use crate::gateway::schema::opcodes::OpCodes::InvalidSession;
use crate::gateway::schema::GatewayMessage;
use crate::state::ThreadData;
use crate::AppState;
use epl_common::nats::Messages;
use crate::gateway::dispatch::typing::dispatch_typing_start;

pub async fn handle_nats_message(thread_data: &mut ThreadData, msg: Messages, state: &AppState) {
    match msg {
        Messages::InvalidateGatewaySession { session } => {
            if session.eq(thread_data.gateway_state.session_id.as_ref().unwrap()) {
                send_message(
                    thread_data,
                    GatewayMessage {
                        op: InvalidSession,
                        ..Default::default()
                    },
                )
                .await;
            }
        }
        Messages::Error { .. } => {
            todo!();
        }
        Messages::RelationshipAdd { user_id, req_type } => {
            dispatch_relationship_add(thread_data, state, user_id, req_type).await;
        }
        Messages::RelationshipRemove { user_id, req_type } => {
            dispatch_relationship_remove(thread_data, state, user_id, req_type).await;
        }
        Messages::ChannelCreate { id } => {
            dispatch_channel_create(thread_data, state, id).await;
        }
        Messages::MessageCreate { id } => {
            dispatch_message(thread_data, state, DispatchMessageTypes::Create, id).await;
        }
        Messages::MessageUpdate { id } => {
            dispatch_message(thread_data, state, DispatchMessageTypes::Update, id).await;
        }
        Messages::MessageDelete { id, channel_id, guild_id } => {
            dispatch_message_delete(thread_data, id, channel_id, guild_id).await;
        }
        Messages::TypingStarted { channel_id, user_id, timestamp } => {
            dispatch_typing_start(thread_data, user_id, channel_id, timestamp).await;
        }
    }
}
