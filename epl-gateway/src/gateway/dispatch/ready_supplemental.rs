use crate::gateway::dispatch::{assemble_dispatch, DispatchData, send_message};
use crate::gateway::dispatch::DispatchTypes::READY_SUPPLEMENTAL;
use crate::gateway::schema::ready::{MergedPresences, ReadySupplemental};

pub async fn dispatch_ready_supplemental() {
    // TODO: This is all stubbed
    send_message(assemble_dispatch(
        READY_SUPPLEMENTAL,
        DispatchData::READY_SUPPLEMENTAL(ReadySupplemental {
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