use crate::gateway::dispatch;
use crate::gateway::dispatch::{assemble_dispatch, send_close, send_message, DispatchTypes};
use crate::gateway::schema::error_codes::ErrorCode::UnknownError;
use crate::gateway::schema::ready::{
    Consents, ConsentsEntry, OtherUser, PrivateChannel, ReadState, Ready, RelationshipReady,
    Session, SessionClientInfo, Tutorial, UserGuildSettings,
};
use crate::state::ThreadData;
use crate::AppState;
use epl_common::database::auth::{get_all_sessions, get_session_by_token};
use epl_common::database::entities::prelude::{Channel, ChannelMember, Relationship, User, Message, UserSetting};
use epl_common::database::entities::{channel, channel_member, message, relationship, user, user_setting};
use epl_common::options::{EplOptions, Options};
use sea_orm::{Condition, EntityTrait, QueryOrder};
use std::collections::HashSet;

use epl_common::channels::ChannelTypes;
use epl_common::flags::{generate_public_flags, get_user_flags};
use epl_common::{RelationshipType, Stub};
use sea_orm::prelude::*;
use epl_common::protobufs::{generate_user_proto, ProtoType};

pub async fn dispatch_ready(
    thread_data: &mut ThreadData,
    user: epl_common::database::entities::user::Model,
    token: &String,
    state: &AppState,
) {
    // TODO: Stub guild settings until we learn more about them
    let user_guild_settings = UserGuildSettings {
        version: 0,
        partial: false,
        entries: vec![],
    };

    let mut relationships: Vec<RelationshipReady> = vec![];
    let mut other_users: Vec<OtherUser> = vec![];

    let mut queued_users: HashSet<i64> = HashSet::new();

    let current_session = match get_session_by_token(&state.conn, token).await {
        Ok(session) => session,
        Err(_) => {
            send_close(thread_data, UnknownError).await;
            return;
        }
    };

    let user_struct = epl_common::User {
        verified: user.acct_verified,
        username: user.username,
        purchased_flags: user.purchased_flags.unwrap_or(0),
        premium_type: user.premium_type.unwrap_or(0),
        premium: (user.premium_type.unwrap_or(0) != 0),
        phone: user.phone,
        nsfw_allowed: user.nsfw_allowed,
        mobile: matches!(current_session.platform.clone().unwrap_or_default().as_str(), "Discord Android" | "Discord iOS"),
        mfa_enabled: user.mfa_enabled,
        id: user.id.to_string(),
        global_name: user.display_name.clone(),
        flags: user.flags,
        email: user.email,
        display_name: user.display_name,
        discriminator: user.discriminator,
        desktop: matches!(current_session.platform.unwrap_or_default().as_str(), "Discord Client"),
        bio: user.bio.unwrap_or_default(),
        banner_color: user.banner_colour,
        banner: user.banner,
        avatar_decoration: user.avatar_decoration,
        avatar: user.avatar,
        accent_color: user.accent_color,
    };

    // TODO: not super important but we stub this and suppress tutorial indicators to not be annoying
    let tutorial = Tutorial {
        indicators_suppressed: true,
        indicators_confirmed: vec![],
    };

    let sessions: Vec<Session> = get_all_sessions(&state.conn, &user.id)
        .await
        .into_iter()
        .map(|session| Session {
            status: session.status,
            session_id: session.session_id,
            client_info: SessionClientInfo {
                version: 0,
                os: session.os.unwrap_or_default(),
                client: match session.platform.unwrap_or_default().as_str() {
                    "Discord Client" => String::from("desktop"),
                    "Discord Android" => String::from("mobile"),
                    "Discord iOS" => String::from("mobile"),
                    _ => String::from("web"),
                },
            },
            activities: vec![],
        })
        .collect();

    // TODO: Stub for now
    let read_state = ReadState {
        version: 0,
        partial: false,
        entries: vec![],
    };

    // TODO: Should we track these?
    let consents = Consents {
        personalization: ConsentsEntry { consented: false },
    };

    let created_relationships = Relationship::find()
        .filter(relationship::Column::Creator.eq(user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let peered_relationships = Relationship::find()
        .filter(relationship::Column::Peer.eq(user.id))
        .filter(relationship::Column::RelationshipType.ne(RelationshipType::Blocked as i32))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    // Gather created relationships first
    for i in created_relationships {
        relationships.push(RelationshipReady {
            user_id: i.peer.to_string(),
            _type: i.relationship_type,
            since: i.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            nickname: None,
            id: i.peer.to_string(),
        });

        queued_users.insert(i.peer);
    }

    // Then peered relationships
    for i in peered_relationships {
        // For peered relationships, they will show as outgoing when the peer should see it as incoming
        let normalized_type = if i.relationship_type == (RelationshipType::Outgoing as i32) {
            RelationshipType::Incoming as i32
        } else {
            i.relationship_type
        };

        relationships.push(RelationshipReady {
            user_id: i.creator.to_string(),
            _type: normalized_type,
            since: i.timestamp.and_utc().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            nickname: None,
            id: i.creator.to_string(),
        });

        queued_users.insert(i.creator);
    }

    // Grab all DMs and Group DMs for user
    let channels_in: Vec<channel_member::Model> = ChannelMember::find()
        .filter(channel_member::Column::User.eq(user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let mut private_channels: Vec<PrivateChannel> = vec![];

    for i in channels_in {
        let channel: channel::Model = Channel::find_by_id(i.channel)
            .filter(
                Condition::any()
                    .add(channel::Column::Type.eq(ChannelTypes::DM as i32))
                    .add(channel::Column::Type.eq(ChannelTypes::GroupDM as i32)),
            )
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .expect("Channel user is member of is missing!");

        thread_data.nats_subscriptions.insert(
            format!("{}", channel.id),
            thread_data
                .nats
                .subscribe(format!("{}", channel.id))
                .await
                .expect("Failed to subscribe!")
        );

        let recipient_ids: Vec<String> = ChannelMember::find()
            .filter(channel_member::Column::Channel.eq(channel.id))
            .filter(channel_member::Column::User.ne(user.id))
            .all(&state.conn)
            .await
            .expect("Failed to access database!")
            .into_iter()
            .map(|e| {
                queued_users.insert(e.user);

                e.user.to_string()
            })
            .collect();
        
        let last_message_id: Option<String> = Message::find()
            .filter(message::Column::ChannelId.eq(channel.id))
            .order_by_desc(message::Column::Id)
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .map(|e| e.id.to_string());

        private_channels.push(PrivateChannel {
            _type: channel.r#type,
            recipient_ids,
            last_message_id,
            is_spam: None,
            id: channel.id.to_string(),
            flags: channel.flags.unwrap_or(9),
            owner_id: channel.owner_id.map(|e| e.to_string()),
            name: channel.name,
            icon: channel.icon,
        })
    }

    for i in queued_users {
        let user: user::Model = User::find_by_id(i)
            .one(&state.conn)
            .await
            .expect("Failed to access database!")
            .expect("Missing user while queued!");

        other_users.push(OtherUser {
            username: user.username,
            public_flags: generate_public_flags(get_user_flags(user.flags)),
            id: user.id.to_string(),
            global_name: user.display_name.clone(),
            discriminator: Some(user.discriminator),
            bot: user.bot,
            avatar_decoration: user.avatar_decoration,
            avatar: user.avatar,
        })
    }

    let mut experiments = if EplOptions::get().pomelo {
        vec![[268309827, 0, 1, -1, 7, 1071, 0, 0]]
    } else {
        vec![]
    };

    // Enable sessions
    experiments.push([1095779154, 0, 1, -1, 2, 183, 0, 0]);

    let user_settings = UserSetting::find()
        .filter(user_setting::Column::User.eq(user.id))
        .one(&state.conn)
        .await
        .expect("Failed to access database!")
        .expect("User doesn't have any settings!");

    send_message(
        thread_data,
        assemble_dispatch(DispatchTypes::Ready(Box::from(Ready {
            version: 9,
            users: other_users,
            user_settings_proto: generate_user_proto(ProtoType::PreloadedUserSettings(user_settings)),
            user_guild_settings,
            user: user_struct,
            tutorial,
            sessions,
            // TODO: Need more research about this
            session_type: String::from("normal"),
            session_id: thread_data.gateway_state.session_id.clone().unwrap(),
            resume_gateway_url: EplOptions::get().gateway_url,
            relationships,
            read_state,
            private_channels,
            merged_members: vec![],
            guilds: vec![],
            guild_join_requests: vec![],
            guild_experiments: vec![],
            geo_ordered_rtc_regions: vec![],
            friend_suggestion_count: 0,
            experiments,
            country_code: String::from("US"),
            consents,
            connected_accounts: vec![],
            // TODO: do we really need to hash these?
            auth_session_id_hash: current_session.session_id,
            api_code_version: 1,
            // We don't do analytics
            analytics_token: String::from(""),
            notification_settings: Some(Stub {}),
        }))),
    )
    .await;

    dispatch::ready_supplemental::dispatch_ready_supplemental(thread_data).await;
}
