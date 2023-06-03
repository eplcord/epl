use async_nats::Client;
use epl_common::nats::Messages;

pub async fn send_nats_message(nats_client: &Client, subject: String, message: Messages) {
    nats_client.publish(
        subject,
        serde_json::to_vec(&message)
            .expect("Failed to parse message into json!")
            .into()
    ).await.expect("Failed to send NATS message!");
}