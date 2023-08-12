use crate::gateway::dispatch::{assemble_dispatch, send_message, DispatchTypes};
use crate::gateway::schema::message::{generate_message_struct, generate_refed_message, MessageDelete};
use crate::state::ThreadData;
use crate::AppState;
use epl_common::database::entities::prelude::{Message, User};

use epl_common::database::entities::{message, user};
use sea_orm::prelude::*;

pub enum DispatchMessageTypes {
    Create,
    Update
}

pub async fn dispatch_message(thread_data: &mut ThreadData, state: &AppState, dispatch_type: DispatchMessageTypes, id: i64) {
    let message = Message::find_by_id(id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Failed to get message requested by NATS!");

    let message_author = User::find_by_id(message.author.unwrap_or(0))
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    let mut refed_message: Option<(message::Model, Option<user::Model>)> = None;

    if message.reference_message_id.is_some() {
        refed_message = generate_refed_message(&state.conn, message.reference_message_id.unwrap()).await;
    }

    let dispatch = generate_message_struct(
        message,
        message_author,
        refed_message
    );

    send_message(
        thread_data,
        assemble_dispatch(
            match dispatch_type {
                DispatchMessageTypes::Create => DispatchTypes::MessageCreate(dispatch),
                DispatchMessageTypes::Update => DispatchTypes::MessageUpdate(dispatch),
            }
        ),
    )
    .await;
}


pub async fn dispatch_message_delete(thread_data: &mut ThreadData, id: i64, channel_id: i64, guild_id: Option<i64>)  {
    send_message(
        thread_data,
        assemble_dispatch(
            DispatchTypes::MessageDelete(
                MessageDelete {
                    channel_id: channel_id.to_string(),
                    guild_id: guild_id.map(|e| e.to_string()),
                    id: id.to_string(),
                }
            )
        ),
    ).await;
}