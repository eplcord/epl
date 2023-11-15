use std::collections::HashSet;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::channels::ChannelTypes;
use crate::database::entities::{channel, message, user};
use crate::database::entities::prelude::*;

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