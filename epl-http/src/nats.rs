use crate::AppState;
use async_nats::Client;
use epl_common::nats::Messages;
use epl_common::RelationshipType::{Blocked, Friend, Incoming, Outgoing};

pub async fn send_nats_message(nats_client: &Client, subject: String, message: Messages) {
    nats_client
        .publish(
            subject,
            serde_json::to_vec(&message)
                .expect("Failed to parse message into json!")
                .into(),
        )
        .await
        .expect("Failed to send NATS message!");
}

pub enum RelationshipUpdate {
    Create,
    Accept,
    Block,
    Remove,
}

pub async fn send_relationship_update(
    state: &AppState,
    creator: i64,
    peer: i64,
    instruction: RelationshipUpdate,
) {
    match instruction {
        RelationshipUpdate::Create => {
            send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipAdd {
                    user_id: peer,
                    req_type: Incoming,
                },
            )
            .await;

            send_nats_message(
                &state.nats_client,
                peer.to_string(),
                Messages::RelationshipAdd {
                    user_id: creator,
                    req_type: Outgoing,
                },
            )
            .await;
        }
        RelationshipUpdate::Remove => {
            send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipRemove {
                    user_id: peer,
                    req_type: Outgoing,
                },
            )
            .await;

            send_nats_message(
                &state.nats_client,
                peer.to_string(),
                Messages::RelationshipRemove {
                    user_id: creator,
                    req_type: Incoming,
                },
            )
            .await;
        }
        RelationshipUpdate::Block => {
            send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipAdd {
                    user_id: peer,
                    req_type: Blocked,
                },
            )
            .await;
        }
        RelationshipUpdate::Accept => {
            send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipAdd {
                    user_id: peer,
                    req_type: Friend,
                },
            )
            .await;

            send_nats_message(
                &state.nats_client,
                peer.to_string(),
                Messages::RelationshipAdd {
                    user_id: creator,
                    req_type: Friend,
                },
            )
            .await;
        }
    }
}
