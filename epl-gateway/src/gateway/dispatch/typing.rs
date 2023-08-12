use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use epl_common::Stub;
use crate::gateway::dispatch::{assemble_dispatch, DispatchTypes, send_message};
use crate::state::ThreadData;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone)]
pub struct TypingStart {
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub user_id: String,
    pub timestamp: i64,
    // FIXME: Implement once guilds are a thing
    pub member: Option<Stub>
}

pub async fn dispatch_typing_start(
    thread_data: &mut ThreadData,
    user_id: i64,
    channel_id: i64,
    timestamp: NaiveDateTime
) {
    // Skip sending the event if we're the ones typing
    if user_id.eq(&thread_data.gateway_state.user_id.unwrap()) {
        return;
    }

    send_message(
        thread_data,
        assemble_dispatch(
            DispatchTypes::TypingStart(
                TypingStart {
                    channel_id: channel_id.to_string(),
                    // FIXME: Implement this when guilds are a thing
                    guild_id: None,
                    user_id: user_id.to_string(),
                    timestamp: timestamp.timestamp(),
                    // FIXME: Implement this when guilds are a thing
                    member: None,
                }
            )
        ),
    ).await;
}