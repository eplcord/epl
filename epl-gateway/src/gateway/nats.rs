use crate::gateway::dispatch::channel::{ChannelRecipientUpdateType, dispatch_channel_create, dispatch_channel_delete, dispatch_channel_recipient_update};
use crate::gateway::dispatch::message::{dispatch_message, dispatch_message_delete, DispatchMessageTypes};
use crate::gateway::dispatch::relationships::{
    dispatch_relationship_add, dispatch_relationship_remove,
};
use crate::gateway::dispatch::{send_close, send_message};
use crate::gateway::schema::opcodes::OpCodes::InvalidSession;
use crate::gateway::schema::GatewayMessage;
use crate::state::ThreadData;
use crate::AppState;
use epl_common::nats::Messages;
use crate::gateway::dispatch::typing::dispatch_typing_start;
use crate::gateway::dispatch::user_note_update::dispatch_user_note_update;
use crate::gateway::schema::error_codes::ErrorCode;

pub async fn handle_nats_message(thread_data: &mut ThreadData, msg: Messages, state: &AppState) {
    match msg {
        Messages::InvalidateGatewaySession { session } => {
            if session.eq(thread_data.gateway_state.session_id.as_ref().unwrap()) || session.eq("all") {
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
            send_close(thread_data, ErrorCode::UnknownError).await;
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
        Messages::ChannelDelete { id } => {
            dispatch_channel_delete(thread_data, state, id).await;
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
        Messages::ChannelRecipientAdd { channel_id, user_id } => {
            dispatch_channel_recipient_update(thread_data, state, channel_id, user_id, ChannelRecipientUpdateType::Add).await;
        }
        Messages::ChannelRecipientRemove { channel_id, user_id } => {
            dispatch_channel_recipient_update(thread_data, state, channel_id, user_id, ChannelRecipientUpdateType::Remove).await;
        }
        Messages::UserNoteUpdate { creator_id, subject_id } => {
            dispatch_user_note_update(thread_data, state, creator_id, subject_id).await;
        }
    }
}
