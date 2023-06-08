use crate::state::ThreadData;
use crate::AppState;
use epl_common::database::entities::{channel, channel_member};

use crate::gateway::dispatch::{assemble_dispatch, send_message, DispatchTypes};
use crate::gateway::schema::channels::ChannelCreate;
use crate::gateway::schema::SharedUser;
use epl_common::channels::ChannelTypes;
use epl_common::database::entities::prelude::{Channel, ChannelMember, User};
use epl_common::flags::{generate_public_flags, get_user_flags};
use sea_orm::prelude::*;

pub async fn dispatch_channel_create(thread_data: &mut ThreadData, state: &AppState, id: i64) {
    let channel: channel::Model = Channel::find_by_id(id)
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("Channel requested by internal NATS missing!");

    thread_data.nats_subscriptions.push(
        thread_data
            .nats
            .subscribe(format!("{}", channel.id))
            .await
            .expect("Failed to subscribe!")
    );

    // If this is a DM or group DM, we need to provide the users in recipients
    let recipients: Option<Vec<SharedUser>> = if channel.r#type == (ChannelTypes::DM as i32)
        || channel.r#type == (ChannelTypes::GroupDM as i32)
    {
        let mut output: Vec<SharedUser> = vec![];

        let members: Vec<channel_member::Model> = ChannelMember::find()
            .filter(channel_member::Column::Channel.eq(id))
            .all(&state.conn)
            .await
            .expect("Failed to access database!");

        for i in members {
            if i.user.eq(&thread_data.gateway_state.user_id.unwrap()) {
                continue;
            }

            let user = i
                .find_related(User)
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("Invalid user referenced in channel_member!");

            output.push(SharedUser {
                avatar: user.avatar,
                avatar_decoration: user.avatar_decoration,
                discriminator: Some(user.discriminator),
                global_name: None,
                id: user.id.to_string(),
                public_flags: generate_public_flags(get_user_flags(user.flags)),
                username: user.username,
            })
        }

        Some(output)
    } else {
        None
    };

    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::ChannelCreate(ChannelCreate {
            flags: channel.flags.unwrap_or(0),
            guild_id: channel.guild_id.map(|e| e.to_string()),
            id: channel.id.to_string(),
            last_message_id: channel.last_message_id.map(|e| e.to_string()),
            name: channel.name,
            nsfw: channel.nsfw,
            parent_id: channel.parent_id.map(|e| e.to_string()),
            permission_overwrites: None,
            position: channel.position,
            rate_limit_per_user: channel.rate_limit_per_user,
            topic: channel.topic,
            recipients,
            _type: channel.r#type,
            version: None,
            is_spam: Some(false),
        })),
    )
    .await;
}
