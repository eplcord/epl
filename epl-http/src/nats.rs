use crate::AppState;
use epl_common::nats;
use epl_common::nats::Messages;
use epl_common::RelationshipType::{Blocked, Friend, Incoming, Outgoing};

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
            nats::send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipAdd {
                    user_id: peer,
                    req_type: Incoming,
                },
            )
            .await;

            nats::send_nats_message(
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
            nats::send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipRemove {
                    user_id: peer,
                    req_type: Outgoing,
                },
            )
            .await;

            nats::send_nats_message(
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
            nats::send_nats_message(
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
            nats::send_nats_message(
                &state.nats_client,
                creator.to_string(),
                Messages::RelationshipAdd {
                    user_id: peer,
                    req_type: Friend,
                },
            )
            .await;

            nats::send_nats_message(
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
