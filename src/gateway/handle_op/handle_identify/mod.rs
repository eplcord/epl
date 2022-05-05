use rocket::debug;
use rocket::futures::SinkExt;
use rocket::futures::stream::SplitSink;
use warp::ws::{Message, WebSocket};

use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::identify::Identify;

pub async fn handle_identify(data: Identify, write: &mut SplitSink<WebSocket, Message>) {
    debug!("Hello from handle_identify!");

    debug!("User token is: {}", data.token);

    write.send(Message::close_with(ErrorCode::AuthenticationFailed, ErrorCode::AuthenticationFailed))
        .await
        .expect("Failed to close websocket!");
}