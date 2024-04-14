use crate::state::ThreadData;
use crate::AppState;
use epl_common::database::entities::{channel, channel_member, message, pin, user};

use crate::gateway::dispatch::{assemble_dispatch, send_message, DispatchTypes};
use crate::gateway::schema::channels::{ChannelCreate, ChannelDelete, ChannelPinsAck, ChannelPinsUpdate, ChannelRecipientAdd, ChannelRecipientRemove};
use epl_common::channels::ChannelTypes;
use epl_common::database::entities::prelude::{Channel, ChannelMember, Message, Pin, User};
use epl_common::flags::{generate_public_flags, get_user_flags};
use sea_orm::prelude::*;
use sea_orm::QueryOrder;
use epl_common::schema::v9;
use epl_common::schema::v9::user::generate_user_struct;

#[derive(Eq, PartialEq)]
pub enum ChannelTypeUpdate {
    CREATE,
    UPDATE
}

pub async fn dispatch_channel_update(thread_data: &mut ThreadData, state: &AppState, id: i64, update_type: ChannelTypeUpdate) {
    let channel: channel::Model = Channel::find_by_id(id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    if update_type.eq(&ChannelTypeUpdate::CREATE)  {
        thread_data.nats_subscriptions.insert(
            format!("{}", channel.id),
            thread_data
                .nats
                .subscribe(format!("{}", channel.id))
                .await
                .expect("Failed to subscribe!")
        );
    }

    // If this is a DM or group DM, we need to provide the users in recipients
    let recipients: Option<Vec<v9::user::User>> = if channel.r#type == (ChannelTypes::DM as i32)
        || channel.r#type == (ChannelTypes::GroupDM as i32)
    {
        let mut output: Vec<v9::user::User> = vec![];

        let members: Vec<channel_member::Model> = ChannelMember::find()
            .filter(channel_member::Column::Channel.eq(id))
            .all(&state.conn)
            .await
            .expect("Failed to access database!");

        for i in members {
            let user = i
                .find_related(User)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Invalid user referenced in channel_member!");

            output.push(v9::user::User {
                avatar: user.avatar,
                avatar_decoration: user.avatar_decoration,
                discriminator: Some(user.discriminator),
                global_name: user.display_name.clone(),
                id: user.id.to_string(),
                public_flags: generate_public_flags(get_user_flags(user.flags)),
                username: user.username,
            })
        }

        Some(output)
    } else {
        None
    };

    let last_message_id: Option<String> = Message::find()
        .filter(message::Column::ChannelId.eq(channel.id))
        .order_by_desc(message::Column::Id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .map(|e| e.id.to_string());
    
    let channel = ChannelCreate {
        flags: channel.flags.unwrap_or(0),
        guild_id: channel.guild_id.map(|e| e.to_string()),
        id: channel.id.to_string(),
        last_message_id,
        name: channel.name,
        icon: channel.icon,
        nsfw: channel.nsfw,
        parent_id: channel.parent_id.map(|e| e.to_string()),
        permission_overwrites: None,
        position: channel.position,
        rate_limit_per_user: channel.rate_limit_per_user,
        topic: channel.topic,
        owner_id: channel.owner_id.map(|e| e.to_string()),
        recipients,
        _type: channel.r#type,
        version: None,
        is_spam: Some(false),
    };

    send_message(
        thread_data,
        assemble_dispatch(
            match update_type {
                ChannelTypeUpdate::CREATE => DispatchTypes::ChannelCreate(channel),
                ChannelTypeUpdate::UPDATE => DispatchTypes::ChannelUpdate(channel)
            }
        ),
    )
    .await;
}


pub async fn dispatch_channel_delete(thread_data: &mut ThreadData, state: &AppState, id: i64) {
    let channel: channel::Model = Channel::find_by_id(id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    thread_data.nats_subscriptions.remove(&format!("{}", channel.id))
        .unwrap()
        .unsubscribe()
        .await
        .expect("Failed to unsubscribe!");

    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::ChannelDelete(ChannelDelete {
            flags: channel.flags.unwrap_or(0),
            guild_id: channel.guild_id.map(|e| e.to_string()),
            id: channel.id.to_string(),
            last_message_id: channel.last_message_id.map(|e| e.to_string()),
            name: channel.name,
            icon: None,
            owner_id: channel.owner_id.map(|e| e.to_string()),
            _type: channel.r#type,
        })),
    )
    .await;
}

pub enum ChannelRecipientUpdateType {
    Add,
    Remove,
}
pub async fn dispatch_channel_recipient_update(
    thread_data: &mut ThreadData,
    state: &AppState,
    channel_id: i64,
    user_id: i64,
    update_type: ChannelRecipientUpdateType,
) {
    let channel: channel::Model = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    let user: user::Model = User::find_by_id(user_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("User requested by internal NATS missing!");

    send_message(
        thread_data,
        match update_type {
            ChannelRecipientUpdateType::Add => {
                assemble_dispatch(DispatchTypes::ChannelRecipientAdd(
                    ChannelRecipientAdd {
                        channel_id: channel.id.to_string(),
                        user: generate_user_struct(user),
                    },
                ))
            },
            ChannelRecipientUpdateType::Remove => {
                assemble_dispatch(DispatchTypes::ChannelRecipientRemove(
                    ChannelRecipientRemove {
                        channel_id: channel.id.to_string(),
                        user: generate_user_struct(user),
                    },
                ))
            }
        }
    ).await;
}

pub async fn dispatch_channel_pins_update(
    thread_data: &mut ThreadData,
    state: &AppState,
    channel_id: i64,
) {
    let channel: channel::Model = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    let latest_pin: Option<pin::Model> = Pin::find()
        .filter(pin::Column::Channel.eq(channel.id))
        .order_by_desc(pin::Column::Timestamp)
        .one(&state.conn)
        .await
        .expect("Failed to access database!");

    send_message(
        thread_data,
        match latest_pin {
            None => {
                assemble_dispatch(DispatchTypes::ChannelPinsUpdate(
                    ChannelPinsUpdate {
                        last_pin_timestamp: None,
                        channel_id: channel.id.to_string(),
                        guild_id: channel.guild_id.map(|x| x.to_string()),
                    }
                ))
            }
            Some(pin) => {
                assemble_dispatch(DispatchTypes::ChannelPinsUpdate(
                    ChannelPinsUpdate {
                        last_pin_timestamp: Some(pin.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string()),
                        channel_id: channel.id.to_string(),
                        guild_id: channel.guild_id.map(|x| x.to_string()),
                    }
                ))
            }
        }
    ).await;
}

pub async fn dispatch_channel_pins_ack(
    thread_data: &mut ThreadData,
    state: &AppState,
    channel_id: i64,
) {
    let channel: channel::Model = Channel::find_by_id(channel_id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::ChannelPinsAck(
            ChannelPinsAck {
                timestamp: chrono::Utc::now().naive_utc().and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
                channel_id: channel.id.to_string(),
                guild_id: channel.guild_id.map(|x| x.to_string()),
                version: chrono::Utc::now().naive_utc().and_utc().timestamp_millis(),
            }
        ))
    ).await;
}