use std::collections::HashSet;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::channels::ChannelTypes;
use crate::database::entities::{channel, channel_member, message, relationship, user};
use crate::database::entities::prelude::*;
use crate::RelationshipType;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InternalChannelPermissions {
    // Meta permissions
    LeaveChannel,
    ViewChannel,
    CreateChannel,
    CreateInvite,
    DeleteChannel,

    // General text permissions
    ViewHistory,
    SendMessage,
    SendTTSMessage,
    SendVoiceMessage,
    EditMessage,
    DeleteMessage,
    PinMessage,
    EmbedLinks,
    AttachFiles,
    AddReactions,
    UseExternalEmojis,
    UseExternalStickers,
    MentionEveryone,
    UseApplicationCommands,
    UseClyde,

    // DM permissions
    StartCall,

    // Group DM permissions
    EditIcon,

    // Call permissions
    JoinCall,
    UseVoiceActivity,
    DisconnectCallMembers,

    // Channel management permissions
    EditName,
    EditTopic,
    EditPosition,
    EditNSFW,
    EditRateLimit,
    EditPermissionOverwrites,

    // Member management permissions
    KickMembers,
    BanMembers,
    EditMemberRoles,
    EditMemberNickname,
}

// Reasonable defaults for DMs
static DM_DEFAULTS: Lazy<HashSet<InternalChannelPermissions>> = Lazy::new(|| {
    let mut permissions = HashSet::new();

    // Meta permissions
    permissions.insert(InternalChannelPermissions::LeaveChannel);

    // General text permissions
    permissions.insert(InternalChannelPermissions::ViewHistory);
    permissions.insert(InternalChannelPermissions::SendMessage);
    permissions.insert(InternalChannelPermissions::SendTTSMessage);
    permissions.insert(InternalChannelPermissions::SendVoiceMessage);
    permissions.insert(InternalChannelPermissions::PinMessage);
    permissions.insert(InternalChannelPermissions::EmbedLinks);
    permissions.insert(InternalChannelPermissions::AttachFiles);
    permissions.insert(InternalChannelPermissions::AddReactions);
    permissions.insert(InternalChannelPermissions::UseExternalEmojis);
    permissions.insert(InternalChannelPermissions::UseExternalStickers);
    permissions.insert(InternalChannelPermissions::MentionEveryone);
    permissions.insert(InternalChannelPermissions::UseApplicationCommands);
    permissions.insert(InternalChannelPermissions::UseClyde);

    permissions
});

static GROUP_DM_ADDITIONS: Lazy<HashSet<InternalChannelPermissions>> = Lazy::new(|| {
    let mut permissions = HashSet::new();

    // Group DM permissions
    permissions.insert(InternalChannelPermissions::EditIcon);

    // DM permissions
    permissions.insert(InternalChannelPermissions::StartCall);

    // Channel management permissions
    permissions.insert(InternalChannelPermissions::EditName);
    permissions.insert(InternalChannelPermissions::EditTopic);

    permissions
});

pub async fn internal_permission_calculator(
    channel: &channel::Model,
    user: &user::Model,
    message: Option<&message::Model>,
    conn: &DatabaseConnection
) -> HashSet<InternalChannelPermissions> {
    let mut permissions: HashSet<InternalChannelPermissions> = HashSet::new();

    // Check if user is a member of the channel
    let channel_member = ChannelMember::find_by_id((channel.id, user.id))
        .one(conn)
        .await
        .expect("Failed to get channel member");

    if channel_member.is_some() {
        // User is a member of the channel
        permissions.insert(InternalChannelPermissions::ViewChannel);
    } else {
        // User is not a member of the channel, bail out with empty permissions
        return permissions;
    }

    // Match channel type
    match ChannelTypes::from_i32(channel.r#type).expect("Unable to convert type!") {
        ChannelTypes::DM => {
            permissions.extend(DM_DEFAULTS.iter());

            // TODO: See how Discord handles a closed DM and if it reopens it if a message is sent
            // FIXME: This is a hacky way to get the other user in the DM, see if there's a better way
            let channel_member = ChannelMember::find()
                .filter(channel_member::Column::Channel.eq(channel.id))
                .filter(channel_member::Column::User.ne(user.id))
                .one(conn)
                .await
                .expect("Failed to get channel member");

            let relationship = match channel_member {
                Some(channel_member) => {
                    // TODO: Investigate more SQL ways to do this
                    // Litecord also does this so :) blueprints/relationships.py#373
                    let relationship = Relationship::find_by_id((channel_member.user, user.id))
                        .one(conn)
                        .await
                        .expect("Failed to get relationship");

                    if relationship.is_some() {
                        relationship
                    } else {
                        Relationship::find_by_id((user.id, channel_member.user))
                            .one(conn)
                            .await
                            .expect("Failed to get relationship")
                    }
                }
                None => None
            };

            match relationship {
                None => {
                    // TODO: Implement privacy settings
                    // We'll act like they're blocked for now
                    permissions.remove(&InternalChannelPermissions::SendMessage);
                    permissions.remove(&InternalChannelPermissions::AddReactions);
                    permissions.remove(&InternalChannelPermissions::PinMessage);
                    permissions.remove(&InternalChannelPermissions::StartCall);
                }
                Some(relationship) => {
                    // Check if the relationship is blocked
                    if relationship.relationship_type.eq(&(RelationshipType::Blocked as i32)) {
                        // Disable sending messages, adding reactions, pinning messages, and calling
                        permissions.remove(&InternalChannelPermissions::SendMessage);
                        permissions.remove(&InternalChannelPermissions::AddReactions);
                        permissions.remove(&InternalChannelPermissions::PinMessage);
                        permissions.remove(&InternalChannelPermissions::StartCall);
                    }
                }
            }

        }
        ChannelTypes::GroupDM => {
            permissions.extend(DM_DEFAULTS.iter());
            permissions.extend(GROUP_DM_ADDITIONS.iter());

            // Check to see if the user is the owner of the group dm
            if let Some(owner_id) = channel.owner_id {
                if owner_id.eq(&user.id) {
                    // User is the owner of the group dm
                    permissions.insert(InternalChannelPermissions::KickMembers);
                    permissions.insert(InternalChannelPermissions::DeleteChannel);
                }
            }
        }
        _ => {
            unimplemented!("Guild channels are not implemented yet!")
        }
    }

    // Run permission check for self created messages (overrides all other permissions)
    if let Some(message) = message {
        if message.author.eq(&Some(user.id)) {
            permissions.insert(InternalChannelPermissions::DeleteMessage);
            permissions.insert(InternalChannelPermissions::EditMessage);
        }
    }

    permissions
}