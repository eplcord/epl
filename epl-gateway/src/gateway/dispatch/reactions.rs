use sea_orm::{EntityTrait, ModelTrait};
use epl_common::database::entities::{message, reaction};
use epl_common::database::entities::prelude::{Message, Reaction};
use epl_common::schema::v9::message::Emoji;
use crate::AppState;
use crate::gateway::dispatch::{assemble_dispatch, DispatchTypes, send_message};
use crate::gateway::schema::reactions::{MessageReactionAdd, MessageReactionRemove};
use crate::state::ThreadData;

pub async fn dispatch_message_reaction_add(
    thread_data: &mut ThreadData,
    state: &AppState,
    message_id: i64,
    user_id: i64,
    emoji: String,
) {
    let reaction: reaction::Model = Reaction::find_by_id((user_id, message_id, emoji))
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Can't find reaction wanted by internal NATS!");

    let message: message::Model = reaction.find_related(Message)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Can't find message wanted by internal NATS!");

    send_message(
        thread_data,
        assemble_dispatch(
            DispatchTypes::MessageReactionAdd(
                MessageReactionAdd {
                    user_id: user_id.to_string(),
                    _type: if reaction.burst {
                        1
                    } else {
                        0
                    },
                    message_id: message.id.to_string(),
                    message_author_id: message.author.map(|x| x.to_string()),
                    // TODO: Implement this when guilds are a thing
                    member: None,
                    emoji: Emoji {
                        // TODO: Implement this when guilds are a thing
                        id: None,
                        name: reaction.emoji,
                    },
                    channel_id: message.channel_id.to_string(),
                    burst: reaction.burst,
                    // TODO: Implement this when guilds are a thing
                    guild_id: None,
                }
            )
        ),
    ).await;
}

pub async fn dispatch_message_reaction_remove(
    thread_data: &mut ThreadData,
    state: &AppState,
    message_id: i64,
    user_id: i64,
    emoji: String,
) {
    let reaction: reaction::Model = Reaction::find_by_id((user_id, message_id, emoji))
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Can't find reaction wanted by internal NATS!");

    let message: message::Model = reaction.find_related(Message)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Can't find message wanted by internal NATS!");
    
    send_message(
        thread_data,
        assemble_dispatch(
            DispatchTypes::MessageReactionRemove(
                MessageReactionRemove {
                    user_id: user_id.to_string(),
                    _type: if reaction.burst {
                        1
                    } else {
                        0
                    },
                    message_id: message.id.to_string(),
                    // TODO: Implement this when guilds are a thing
                    emoji: Emoji {
                        id: None,
                        name: reaction.emoji,
                    },
                    channel_id: message.channel_id.to_string(),
                    burst: reaction.burst,
                    // TODO: Implement this when guilds are a thing
                    guild_id: None,
                }
            )
        ),
    ).await;
}