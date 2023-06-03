// User and application flags taken from https://flags.lewisakura.moe/

use std::collections::HashSet;
use enum_iterator::{all, Sequence};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

static PUBLIC_FLAGS: Lazy<HashSet<UserFlags>> = Lazy::new(|| {
    let mut set = HashSet::new();

    set.insert(UserFlags::Staff);
    set.insert(UserFlags::Partner);
    set.insert(UserFlags::Hypesquad);
    set.insert(UserFlags::BugHunterLevel1);
    set.insert(UserFlags::HypesquadOnlineHouse1);
    set.insert(UserFlags::HypesquadOnlineHouse2);
    set.insert(UserFlags::HypesquadOnlineHouse3);
    set.insert(UserFlags::PremiumEarlySupporter);
    set.insert(UserFlags::System);
    set.insert(UserFlags::BugHunterLevel2);
    set.insert(UserFlags::VerifiedBot);
    set.insert(UserFlags::VerifiedDeveloper);
    set.insert(UserFlags::CertifiedModerator);
    set.insert(UserFlags::ActiveDeveloper);

    set
});

#[repr(i64)]
#[derive(Hash, Eq, PartialEq, Sequence, Clone, Copy)]
/// Discord User Flags
pub enum UserFlags {
    /// Discord Employee
    Staff = 1 << 0,
    /// Partnered Server Owner
    Partner = 1 << 1,
    /// HypeSquad Events Member (a.k.a Legacy Hypesquad)
    Hypesquad = 1 << 2,
    /// Bug Hunter Level 1
    BugHunterLevel1 = 1 << 3,
    /// SMS MFA Enabled for User
    MFASMS = 1 << 4,
    /// ? Best Guess: User Has Dismissed Nitro Promo
    PremiumPromoDismissed = 1 << 5,
    /// House Bravery Member
    HypesquadOnlineHouse1 = 1 << 6,
    /// House Brilliance Member
    HypesquadOnlineHouse2 = 1 << 7,
    /// House Balance Member
    HypesquadOnlineHouse3 = 1 << 8,
    /// Early Nitro Supporter
    PremiumEarlySupporter = 1 << 9,
    /// User is a Team (https://discord.com/developers/docs/topics/teams)
    TeamPseudoUser = 1 << 10,
    /// ? Best Guess: Related to Partner/Verification Applications
    InternalApplication = 1 << 11,
    /// Discord System User
    System = 1 << 12,
    /// User has Unread Messages from Discord
    HasUnreadUrgentMessages = 1 << 13,
    /// Bug Hunter Level 2
    BugHunterLevel2 = 1 << 14,
    /// User Deleted Due to Underage
    UnderageDeleted = 1 << 15,
    /// Verified Bot
    VerifiedBot = 1 << 16,
    /// Early Verified Bot Developer (Legacy)
    VerifiedDeveloper = 1 << 17,
    /// Moderator Programs Alumni
    CertifiedModerator = 1 << 18,
    /// Bot Only Uses HTTP Interactions
    BotHTTPInteractions = 1 << 19,
    /// User Marked As Possible Spammer
    Spammer = 1 << 20,
    /// Disable Nitro Features for User
    DisablePremium = 1 << 21,
    /// Active Developer (https://support-dev.discord.com/hc/en-us/articles/10113997751447)
    ActiveDeveloper = 1 << 22,
    /// User has High Rate limits
    HighGlobalRateLimit = 1 << 33,
    /// User has Been Deleted
    Deleted = 1 << 34,
    /// User Disabled Due to Suspicious Activity
    DisabledSuspiciousActivity = 1 << 35,
    /// User Deleted their Account
    SelfDeleted = 1 << 36,
    /// ?
    PremiumDiscriminator = 1 << 37,
    /// User has Used the Desktop Client
    UsedDesktopClient = 1 << 38,
    /// User has Used the Web Client
    UsedWebClient = 1 << 39,
    /// User has Used the Mobile Client
    UsedMobileClient = 1 << 40,
    /// User is Disabled
    Disabled = 1 << 41,
    /// User has Verified their Email Address
    VerifiedEmail = 1 << 43,
    /// User is Quarantined and Restricted
    Quarantined = 1 << 44,
    /// ? Best Guess: Has Staff Permissions
    Collaborator = 1 << 50,
    /// ? Best Guess: Has Staff Permissions
    RestrictedCollaborator = 1 << 51
}

#[repr(u64)]
pub enum ApplicationFlags {
    /// Embedded Application is Available to Play
    EmbeddedReleased = 1 << 1,
    /// Application can Create Global Emojis
    ManagedEmoji = 1 << 2,
    /// Application has Ability to Create In-App Purchases
    EmbeddedIAP = 1 << 3,
    /// Application has Permission to Create Group DMs
    GroupDMCreate = 1 << 4,
    /// Application can Access The Local RPC Server
    RPCPrivateBeta = 1 << 5,
    /// ? Best Guess: AutoMod
    ApplicationAutoModerationRuleCreateBadge = 1 << 6,
    /// Application can Create Activity Assets
    AllowAssets = 1 << 8,
    /// Application allows Activity Spectating
    AllowActivityActionSpectate = 1 << 9,
    /// Application can Enable Join Requests
    AllowActivityActionJoinRequest = 1 << 10,
    /// Application has Connected to Local RPC Server Before
    RPCHasConnected = 1 << 11,
    /// Application can Receive PRESENCE_UPDATE Events (>100 Guilds)
    GatewayPresence = 1 << 12,
    /// Application can Receive PRESENCE_UPDATE Events (<100 Guilds)
    GatewayPresenceLimited = 1 << 13,
    /// Application can Receive Member Related Events (>100 Guilds)
    GatewayGuildMembers = 1 << 14,
    /// Application can Receive Member Related Events (<100 Guilds)
    GatewayGuildMembersLimited = 1 << 15,
    /// Application has Suspicious Growth
    VerificationPendingGuildLimit = 1 << 16,
    /// Application is Embedded Within the Discord Client
    Embedded = 1 << 17,
    /// Application can Receive Message Content (>100 Guilds)
    GatewayMessageContent = 1 << 18,
    /// Application can Receive Message Content (<100 Guilds)
    GatewayMessageContentLimited = 1 << 19,
    /// Application is Embedded Within the Discord Client and is By Discord
    EmbeddedFirstParty = 1 << 20,
    /// Application has Global Application Commands
    ApplicationCommandBadge = 1 << 23,
    /// Application is Active (Any Global Command Executed Within the Past 30 Days)
    Active = 1 << 24,
}

#[derive(Serialize, Deserialize)]
pub struct Badge {
    description: String,
    icon: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<String>
}

impl From<UserFlags> for Option<Badge> {
    fn from(value: UserFlags) -> Self {
        match value {
            UserFlags::Staff => Some(Badge {
                description: "Discord Staff".to_string(),
                icon: "5e74e9b61934fc1f67c65515d1f7e60d".to_string(),
                id: "staff".to_string(),
                link: Some("https://discord.com/company".to_string()),
            }),
            UserFlags::Partner => Some(Badge {
                description: "Partnered Server Owner".to_string(),
                icon: "3f9748e53446a137a052f3454e2de41e".to_string(),
                id: "partner".to_string(),
                link: Some("https://discord.com/partners".to_string()),
            }),
            UserFlags::Hypesquad => Some(Badge {
                description: "HypeSquad Events".to_string(),
                icon: "bf01d1073931f921909045f3a39fd264".to_string(),
                id: "hypesquad".to_string(),
                link: Some("https://discord.com/hypesquad".to_string()),
            }),
            UserFlags::BugHunterLevel1 => Some(Badge {
                description: "Discord Bug Hunter".to_string(),
                icon: "2717692c7dca7289b35297368a940dd0".to_string(),
                id: "bug_hunter_level_1".to_string(),
                link: Some("https://support.discord.com/hc/en-us/articles/360046057772-Discord-Bugs".to_string()),
            }),
            UserFlags::HypesquadOnlineHouse1 => Some(Badge {
                description: "HypeSquad Bravery".to_string(),
                icon: "8a88d63823d8a71cd5e390baa45efa02".to_string(),
                id: "hypesquad_house_1".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::HypesquadOnlineHouse2 => Some(Badge {
                description: "HypeSquad Brilliance".to_string(),
                icon: "011940fd013da3f7fb926e4a1cd2e618".to_string(),
                id: "hypesquad_house_2".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::HypesquadOnlineHouse3 => Some(Badge {
                description: "HypeSquad Balance".to_string(),
                icon: "3aa41de486fa12454c3761e8e223442e".to_string(),
                id: "hypesquad_house_3".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::PremiumEarlySupporter => Some(Badge {
                description: "Early Supporter".to_string(),
                icon: "7060786766c9c840eb3019e725d2b358".to_string(),
                id: "early_supporter".to_string(),
                link: Some("https://discord.com/settings/premium".to_string()),
            }),
            UserFlags::BugHunterLevel2 => Some(Badge {
                description: "Discord Bug Hunter".to_string(),
                icon: "848f79194d4be5ff5f81505cbd0ce1e6".to_string(),
                id: "bug_hunter_level_2".to_string(),
                link: Some("https://support.discord.com/hc/en-us/articles/360046057772-Discord-Bugs".to_string()),
            }),
            UserFlags::VerifiedDeveloper => Some(Badge {
                description: "Early Verified Bot Developer".to_string(),
                icon: "6df5892e0f35b051f8b61eace34f4967".to_string(),
                id: "verified_developer".to_string(),
                link: None,
            }),
            UserFlags::CertifiedModerator => Some(Badge {
                description: "Moderator Programmes Alumni".to_string(),
                icon: "fee1624003e2fee35cb398e125dc479b".to_string(),
                id: "certified_moderator".to_string(),
                link: Some("https://discord.com/safety".to_string()),
            }),
            UserFlags::ActiveDeveloper => Some(Badge {
                description: "Active Developer".to_string(),
                icon: "6bdc42827a38498929a4920da12695d9".to_string(),
                id: "active_developer".to_string(),
                link: Some("https://support-dev.discord.com/hc/en-us/articles/10113997751447?ref=badge".to_string()),
            }),
            _ => None,
        }
    }
}

pub fn get_user_flags(user_flags: i64) -> Vec<UserFlags> {
    let mut flags = vec![];

    for i in all::<UserFlags>().collect::<Vec<_>>() {
        if (i as i64 & user_flags) != 0 {
            flags.push(i);
        }
    }

    flags
}

pub fn generate_public_flags(user_flags: Vec<UserFlags>) -> i64 {
    let mut flags = 0;

    for i in user_flags {
        if PUBLIC_FLAGS.contains(&i) {
            flags += i as i64;
        }
    }

    flags
}