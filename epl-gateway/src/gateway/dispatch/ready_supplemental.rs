use epl_common::options::{EplOptions, Options};
use crate::gateway::dispatch::{assemble_dispatch, send_message, DispatchTypes};
use crate::gateway::schema::ready::{MergedPresences, ReadySupplemental};
use crate::state::ThreadData;

pub async fn dispatch_ready_supplemental(thread_data: &mut ThreadData) {
    let mut disclose: Vec<String> = vec![];
    
    if EplOptions::get().pomelo {
        disclose.push("pomelo".to_string())
    }
    
    // TODO: This is all stubbed
    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::ReadySupplemental(ReadySupplemental {
            disclose,
            guilds: vec![],
            lazy_private_channels: vec![],
            merged_members: vec![],
            merged_presences: MergedPresences {
                friends: vec![],
                guilds: vec![],
            },
        })),
    )
    .await;
}
