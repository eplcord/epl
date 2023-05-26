use axum::extract::ws::Message;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::opcodes::{DispatchData, GatewayData, OpCodes};
use crate::gateway::schema::ready::{MergedPresences, ReadySupplemental};
use crate::state::SOCKET;

pub async fn dispatch_ready_supplemental() {
    let mut socket = SOCKET.get().lock().await;

    // TODO: This is all stubbed
    socket
        .as_mut()
        .unwrap()
        .inner
        .send(Message::Text(
            serde_json::to_string(&GatewayMessage {
                s: None,
                t: None,
                op: OpCodes::DISPATCH,
                d: Some(GatewayData::DISPATCH {
                    data: Box::new(DispatchData::READY_SUPPLEMENTAL(ReadySupplemental {
                        disclose: vec![],
                        guilds: vec![],
                        lazy_private_channels: vec![],
                        merged_members: vec![],
                        merged_presences: MergedPresences {
                            friends: vec![],
                            guilds: vec![]
                        },
                    }))
                }),
            })
                .expect("Failed to serialize READY_SUPPLEMENTAL!")
        ))
        .await
        .expect("Failed to send READY_SUPPLEMENTAL to client!");

    drop(socket);
}