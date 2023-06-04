use async_nats::Client;
use epl_common::nats::Messages;
use crate::AppState;

pub async fn send_nats_message(nats_client: &Client, subject: String, message: Messages) {
    nats_client.publish(
        subject,
        serde_json::to_vec(&message)
            .expect("Failed to parse message into json!")
            .into()
    ).await.expect("Failed to send NATS message!");
}

pub enum RelationshipUpdate {
    Create,
    Remove
}

pub async fn send_relationship_update(state: &AppState, a: i64, b: i64, instruction: RelationshipUpdate) {
    match instruction {
        RelationshipUpdate::Create => {
            // To
            send_nats_message(
                &state.nats_client, a.to_string(),
                Messages::RelationshipAdd {
                    originator: b,
                    req_type: 3
                }
            ).await;

            // From
            send_nats_message(
                &state.nats_client, b.to_string(),
                Messages::RelationshipAdd {
                    originator: a,
                    req_type: 4
                }
            ).await;
        }
        RelationshipUpdate::Remove => {
            // To
            send_nats_message(
                &state.nats_client, a.to_string(),
                Messages::RelationshipRemove {
                    originator: b,
                    req_type: 3
                }
            ).await;

            // From
            send_nats_message(
                &state.nats_client, b.to_string(),
                Messages::RelationshipRemove {
                    originator: a,
                    req_type: 4
                }
            ).await;
        }
    }
}