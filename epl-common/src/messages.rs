use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[repr(i32)]
pub enum MessageTypes {
    Default = 0,
    /// Indication that a user has joined the group DM
    RecipientAdd = 1,
    /// Indication that a user has left the group DM
    RecipientRemove = 2,
    /// Indication that a user has started a call
    Call = 3,
    /// Indication that the channel name has changed
    ChannelNameChange = 4,
    /// Indication that the channel icon has changed
    ChannelIconChange = 5,
    /// Indication that someone has pinned a message
    ChannelPinnedMessage = 6,
    /// Indication that someone has joined a guild
    UserJoin = 7,
    /// Indication that someone has boosted a guild
    GuildBoost = 8,
    /// ? Best Guess: Indication that someone has boosted a guild to tier 1
    GuildBoostTier1 = 9,
    /// ? Best Guess: Indication that someone has boosted a guild to tier 2
    GuildBoostTier2 = 10,
    /// ? Best Guess: Indication that someone has boosted a guild to tier 13
    GuildBoostTier3 = 11,
    /// Indication that a channel has been set to follow another channel
    ChannelFollowAdd = 12,
    /// ?
    GuildDiscoveryDisqualified = 14,
    /// ?
    GuildDiscoveryRequalified = 15,
    /// ?
    GuildDiscoveryGracePeriodInitalWarning = 16,
    /// ?
    GuildDiscoveryGracePeriodFinalWarning = 17,
    /// Indication that someone has created a thread
    ThreadCreated = 18,
    /// Reply to another message
    Reply = 19,
    /// Response to slash command
    ChatInputCommand = 20,
    /// ? Best Guess: First message in a thread
    ThreadStarterMessage = 21,
    /// ? Best Guess: Message from Clyde in old guild invite tutorial
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
    AutoModerationAction = 24,
    RoleSubscriptionPurchase = 25,
    InteractionPremiumUpsell = 26,
    StageStart = 27,
    StageEnd = 28,
    StageSpeaker = 29,
    StageTopic = 31,
    GuildApplicationPremiumSubscription = 32,
}
