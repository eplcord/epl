use crate::gateway::dispatch::{assemble_dispatch, DispatchTypes, send_message};
use crate::gateway::schema::ready::{MergedPresences, ReadySupplemental};
use crate::state::ThreadData;

pub async fn dispatch_ready_supplemental(thread_data: &mut ThreadData) {
    // TODO: This is all stubbed
    send_message(thread_data, assemble_dispatch(
        DispatchTypes::ReadySupplemental(ReadySupplemental {
            disclose: vec![],
            guilds: vec![],
            lazy_private_channels: vec![],
            merged_members: vec![],
            merged_presences: MergedPresences {
                friends: vec![],
                guilds: vec![]
            },
        })
    )).await;
}