// User and application flags taken from https://flags.lewisakura.moe/

use std::collections::HashSet;
use enum_iterator::{all, Sequence};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

static PUBLICFLAGS: Lazy<HashSet<UserFlags>> = Lazy::new(|| {
    let mut set = HashSet::new();

    set.insert(UserFlags::STAFF);
    set.insert(UserFlags::PARTNER);
    set.insert(UserFlags::HYPESQUAD);
    set.insert(UserFlags::BUG_HUNTER_LEVEL_1);
    set.insert(UserFlags::HYPESQUAD_ONLINE_HOUSE_1);
    set.insert(UserFlags::HYPESQUAD_ONLINE_HOUSE_2);
    set.insert(UserFlags::HYPESQUAD_ONLINE_HOUSE_3);
    set.insert(UserFlags::PREMIUM_EARLY_SUPPORTER);
    set.insert(UserFlags::SYSTEM);
    set.insert(UserFlags::BUG_HUNTER_LEVEL_2);
    set.insert(UserFlags::VERIFIED_BOT);
    set.insert(UserFlags::VERIFIED_DEVELOPER);
    set.insert(UserFlags::CERTIFIED_MODERATOR);
    set.insert(UserFlags::ACTIVE_DEVELOPER);

    set
});

#[repr(i64)]
#[derive(Hash, Eq, PartialEq, Sequence, Clone, Copy)]
pub enum UserFlags {
    STAFF = 1 << 0,
    PARTNER = 1 << 1,
    HYPESQUAD = 1 << 2,
    BUG_HUNTER_LEVEL_1 = 1 << 3,
    MFA_SMS = 1 << 4,
    PREMIUM_PROMO_DISMISSED = 1 << 5,
    HYPESQUAD_ONLINE_HOUSE_1 = 1 << 6,
    HYPESQUAD_ONLINE_HOUSE_2 = 1 << 7,
    HYPESQUAD_ONLINE_HOUSE_3 = 1 << 8,
    PREMIUM_EARLY_SUPPORTER = 1 << 9,
    TEAM_PSEUDO_USER = 1 << 10,
    INTERNAL_APPLICATION = 1 << 11,
    SYSTEM = 1 << 12,
    HAS_UNREAD_URGENT_MESSAGES = 1 << 13,
    BUG_HUNTER_LEVEL_2 = 1 << 14,
    UNDERAGE_DELETED = 1 << 15,
    VERIFIED_BOT = 1 << 16,
    VERIFIED_DEVELOPER = 1 << 17,
    CERTIFIED_MODERATOR = 1 << 18,
    BOT_HTTP_INTERACTIONS = 1 << 19,
    SPAMMER = 1 << 20,
    DISABLE_PREMIUM = 1 << 21,
    ACTIVE_DEVELOPER = 1 << 22,
    HIGH_GLOBAL_RATE_LIMIT = 1 << 33,
    DELETED = 1 << 34,
    DISABLED_SUSPICIOUS_ACTIVITY = 1 << 35,
    SELF_DELETED = 1 << 36,
    PREMIUM_DISCRIMINATOR = 1 << 37,
    USED_DESKTOP_CLIENT = 1 << 38,
    USED_WEB_CLIENT = 1 << 39,
    USED_MOBILE_CLIENT = 1 << 40,
    DISABLED = 1 << 41,
    VERIFIED_EMAIL = 1 << 43,
    QUARANTINED = 1 << 44,
    COLLABORATOR = 1 << 50,
    RESTRICTED_COLLABORATOR = 1 << 51
}

#[repr(u64)]
pub enum ApplicationFlags {
    EMBEDDED_RELEASED = 1 << 1,
    MANAGED_EMOJI = 1 << 2,
    EMBEDDED_IAP = 1 << 3,
    GROUP_DM_CREATE = 1 << 4,
    RPC_PRIVATE_BETA = 1 << 5,
    APPLICATION_AUTO_MODERATION_RULE_CREATE_BADGE = 1 << 6,
    ALLOW_ASSETS = 1 << 8,
    ALLOW_ACTIVITY_ACTION_SPECTATE = 1 << 9,
    ALLOW_ACTIVITY_ACTION_JOIN_REQUEST = 1 << 10,
    RPC_HAS_CONNECTED = 1 << 11,
    GATEWAY_PRESENCE = 1 << 12,
    GATEWAY_PRESENCE_LIMITED = 1 << 13,
    GATEWAY_GUILD_MEMBERS = 1 << 14,
    GATEWAY_GUILD_MEMBERS_LIMITED = 1 << 15,
    VERIFICATION_PENDING_GUILD_LIMIT = 1 << 16,
    EMBEDDED = 1 << 17,
    GATEWAY_MESSAGE_CONTENT = 1 << 18,
    GATEWAY_MESSAGE_CONTENT_LIMITED = 1 << 19,
    EMBEDDED_FIRST_PARTY = 1 << 20,
    APPLICATION_COMMAND_BADGE = 1 << 23,
    ACTIVE = 1 << 24,
}

#[derive(Serialize, Deserialize)]
pub struct Badge {
    description: String,
    icon: String,
    id: String,
    link: Option<String>
}

impl From<UserFlags> for Option<Badge> {
    fn from(value: UserFlags) -> Self {
        match value {
            UserFlags::STAFF => Some(Badge {
                description: "Discord Staff".to_string(),
                icon: "5e74e9b61934fc1f67c65515d1f7e60d".to_string(),
                id: "staff".to_string(),
                link: Some("https://discord.com/company".to_string()),
            }),
            UserFlags::PARTNER => Some(Badge {
                description: "Partnered Server Owner".to_string(),
                icon: "3f9748e53446a137a052f3454e2de41e".to_string(),
                id: "partner".to_string(),
                link: Some("https://discord.com/partners".to_string()),
            }),
            UserFlags::HYPESQUAD => Some(Badge {
                description: "HypeSquad Events".to_string(),
                icon: "bf01d1073931f921909045f3a39fd264".to_string(),
                id: "hypesquad".to_string(),
                link: Some("https://discord.com/hypesquad".to_string()),
            }),
            UserFlags::BUG_HUNTER_LEVEL_1 => Some(Badge {
                description: "Discord Bug Hunter".to_string(),
                icon: "2717692c7dca7289b35297368a940dd0".to_string(),
                id: "bug_hunter_level_1".to_string(),
                link: Some("https://support.discord.com/hc/en-us/articles/360046057772-Discord-Bugs".to_string()),
            }),
            UserFlags::HYPESQUAD_ONLINE_HOUSE_1 => Some(Badge {
                description: "HypeSquad Bravery".to_string(),
                icon: "8a88d63823d8a71cd5e390baa45efa02".to_string(),
                id: "hypesquad_house_1".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::HYPESQUAD_ONLINE_HOUSE_2 => Some(Badge {
                description: "HypeSquad Brilliance".to_string(),
                icon: "011940fd013da3f7fb926e4a1cd2e618".to_string(),
                id: "hypesquad_house_2".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::HYPESQUAD_ONLINE_HOUSE_3 => Some(Badge {
                description: "HypeSquad Balance".to_string(),
                icon: "3aa41de486fa12454c3761e8e223442e".to_string(),
                id: "hypesquad_house_3".to_string(),
                link: Some("https://discord.com/settings/hypesquad-online".to_string()),
            }),
            UserFlags::PREMIUM_EARLY_SUPPORTER => Some(Badge {
                description: "Early Supporter".to_string(),
                icon: "7060786766c9c840eb3019e725d2b358".to_string(),
                id: "early_supporter".to_string(),
                link: Some("https://discord.com/settings/premium".to_string()),
            }),
            UserFlags::BUG_HUNTER_LEVEL_2 => Some(Badge {
                description: "Discord Bug Hunter".to_string(),
                icon: "848f79194d4be5ff5f81505cbd0ce1e6".to_string(),
                id: "bug_hunter_level_2".to_string(),
                link: Some("https://support.discord.com/hc/en-us/articles/360046057772-Discord-Bugs".to_string()),
            }),
            UserFlags::VERIFIED_DEVELOPER => Some(Badge {
                description: "Early Verified Bot Developer".to_string(),
                icon: "6df5892e0f35b051f8b61eace34f4967".to_string(),
                id: "verified_developer".to_string(),
                link: None,
            }),
            UserFlags::CERTIFIED_MODERATOR => Some(Badge {
                description: "Moderator Programmes Alumni".to_string(),
                icon: "fee1624003e2fee35cb398e125dc479b".to_string(),
                id: "certified_moderator".to_string(),
                link: Some("https://discord.com/safety".to_string()),
            }),
            UserFlags::ACTIVE_DEVELOPER => Some(Badge {
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
        if PUBLICFLAGS.contains(&i) {
            flags += i as i64;
        }
    }

    flags
}