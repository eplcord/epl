use std::collections::HashMap;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use prost::Message;
use serde_derive::{Deserialize, Serialize};
use crate::database::entities::{frecency, user_setting};
use crate::protobufs::discord_protos::discord_users::v1;
use crate::protobufs::discord_protos::discord_users::v1::FavoriteGif;

pub mod discord_protos {
    pub mod discord_users {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/discord_protos.discord_users.v1.rs"));
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FrecencyItem {
    pub total_uses: u32,
    pub recent_uses: Vec<u64>,
    pub frecency: i32,
    pub score: i32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FavoriteGIF {
    pub format: GIFType,
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub order: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum GIFType {
    NONE = 0,
    IMAGE = 1,
    VIDEO = 2,
}

impl From<i32> for GIFType {
    fn from(value: i32) -> Self {
        match value {
            0 => GIFType::NONE,
            1 => GIFType::IMAGE,
            2 => GIFType::VIDEO,
            _ => GIFType::NONE
        }
    }
}

pub enum ProtoType {
    PreloadedUserSettings(user_setting::Model),
    FrecencyUserSettings(frecency::Model)
}

pub fn generate_user_proto(proto: ProtoType) -> String {
    let mut buffer = vec![];

    match proto {
        ProtoType::PreloadedUserSettings(settings) => {
            v1::PreloadedUserSettings {
                versions: Some(v1::Version {
                    client_version: 0,
                    server_version: 0,
                    data_version: settings.data_version as u32,
                }),
                inbox: Some(v1::InboxSettings {
                    current_tab: settings.inbox_current_tab,
                    viewed_tutorial: settings.inbox_viewed_tutorial,
                }),
                // TODO: implement when guilds
                guilds: None,
                user_content: Some(v1::UserContentSettings {
                    dismissed_contents: settings.user_content_dismissed.unwrap_or_default().iter().map(|x| *x as u8).collect(),
                    last_dismissed_outbound_promotion_start_date: None,
                    premium_tier_0_modal_dismissed_at: None,
                }),
                voice_and_video: Some(v1::VoiceAndVideoSettings {
                    blur: Some(v1::VideoFilterBackgroundBlur {
                        use_blur: settings.voice_video_background_blur,
                    }),
                    preset_option: settings.voice_video_preset_option as u32,
                    custom_asset: None,
                    always_preview_video: Some(v1::AlwaysPreviewVideo {
                        value: settings.voice_video_always_preview,
                    }),
                    afk_timeout: Some(v1::AfkTimeout {
                        value: settings.voice_afk_timeout as u32,
                    }),
                    stream_notifications_enabled: Some(v1::StreamNotificationsEnabled {
                        value: settings.voice_stream_notifications_enabled,
                    }),
                    native_phone_integration_enabled: Some(v1::NativePhoneIntegrationEnabled {
                        value: settings.voice_native_phone_integration_enabled,
                    }),
                }),
                text_and_images: Some(v1::TextAndImagesSettings {
                    diversity_surrogate: None,
                    use_rich_chat_input: Some(v1::UseRichChatInput {
                        value: settings.text_use_rich_chat_input,
                    }),
                    use_thread_sidebar: Some(v1::UseThreadSidebar {
                        value: settings.text_use_thread_sidebar,
                    }),
                    render_spoilers: Some(v1::RenderSpoilers {
                        value: settings.text_render_spoilers,
                    }),
                    emoji_picker_collapsed_sections: settings.text_emoji_picker_collapsed_sections,
                    sticker_picker_collapsed_sections: settings.text_sticker_picker_collapsed_sections,
                    view_image_descriptions: Some(v1::ViewImageDescriptions {
                        value: settings.text_view_image_descriptions,
                    }),
                    show_command_suggestions: Some(v1::ShowCommandSuggestions {
                        value: settings.text_show_command_suggestions,
                    }),
                    inline_attachment_media: Some(v1::InlineAttachmentMedia {
                        value: settings.text_inline_attachment_media,
                    }),
                    inline_embed_media: Some(v1::InlineEmbedMedia {
                        value: settings.text_inline_embed_media,
                    }),
                    gif_auto_play: Some(v1::GifAutoPlay {
                        value: settings.text_gif_auto_play,
                    }),
                    render_embeds: Some(v1::RenderEmbeds {
                        value: settings.text_render_embeds,
                    }),
                    render_reactions: Some(v1::RenderReactions {
                        value: settings.text_render_reactions,
                    }),
                    animate_emoji: Some(v1::AnimateEmoji {
                        value: settings.text_animate_emoji,
                    }),
                    animate_stickers: Some(v1::AnimateStickers {
                        value: settings.text_animate_stickers as u32,
                    }),
                    enable_tts_command: Some(v1::EnableTtsCommand {
                        value: settings.text_enable_tts_command,
                    }),
                    message_display_compact: Some(v1::MessageDisplayCompact {
                        value: settings.text_message_display_compact,
                    }),
                    explicit_content_filter: Some(v1::ExplicitContentFilter {
                        value: settings.text_explicit_content_filter as u32,
                    }),
                    view_nsfw_guilds: Some(v1::ViewNsfwGuilds {
                        value: settings.text_view_nsfw_guilds,
                    }),
                    convert_emoticons: Some(v1::ConvertEmoticons {
                        value: settings.text_convert_emoticons,
                    }),
                    expression_suggestions_enabled: Some(v1::ExpressionSuggestionsEnabled {
                        value: settings.text_expression_suggestions_enabled,
                    }),
                    view_nsfw_commands: Some(v1::ViewNsfwCommands {
                        value: settings.text_view_nsfw_commands,
                    }),
                }),
                notifications: Some(v1::NotificationSettings {
                    show_in_app_notifications: Some(v1::ShowInAppNotifications {
                        value: settings.notification_show_in_app_notifications,
                    }),
                    notify_friends_on_go_live: Some(v1::NotifyFriendsOnGoLive {
                        value: settings.notification_notify_friends_on_go_live,
                    }),
                    notification_center_acked_before_id: 0,
                }),
                privacy: Some(v1::PrivacySettings {
                    allow_activity_party_privacy_friends: Some(v1::AllowActivityPartyPrivacyFriends {
                        value: settings.privacy_allow_activity_party_privacy_friends,
                    }),
                    allow_activity_party_privacy_voice_channel: Some(v1::AllowActivityPartyPrivacyVoiceChannel {
                        value: settings.privacy_allow_activity_party_privacy_voice_channel,
                    }),
                    restricted_guild_ids: settings.privacy_restricted_guild_ids.unwrap_or_default().iter().map(|x| *x as u64).collect(),
                    default_guilds_restricted: settings.privacy_default_guilds_restricted,
                    allow_accessibility_detection: settings.privacy_allow_accessibility_detection,
                    detect_platform_accounts: Some(v1::DetectPlatformAccounts {
                        value: settings.privacy_detect_platform_accounts,
                    }),
                    passwordless: None,
                    contact_sync_enabled: Some(v1::ContactSyncEnabled {
                        value: settings.privacy_contact_sync_enabled,
                    }),
                    friend_source_flags: Some(v1::FriendSourceFlags {
                        value: settings.privacy_friend_source_flags as u32,
                    }),
                    friend_discovery_flags: Some(v1::FriendDiscoveryFlags {
                        value: settings.privacy_friend_discovery_flags as u32,
                    }),
                    activity_restricted_guild_ids: settings.privacy_activity_restricted_guild_ids.unwrap_or_default().iter().map(|x| *x as u64).collect(),
                    default_guilds_activity_restricted: settings.privacy_guild_activity_status_restriction_default,
                    activity_joining_restricted_guild_ids: settings.privacy_activity_joining_restricted_guild_ids.unwrap_or_default().iter().map(|x| *x as u64).collect(),
                }),
                debug: Some(v1::DebugSettings {
                    rtc_panel_show_voice_states: Some(v1::RtcPanelShowVoiceStates {
                        value: settings.debug_rtc_panel_show_voice_states,
                    }),
                }),
                game_library: Some(v1::GameLibrarySettings {
                    install_shortcut_desktop: Some(v1::InstallShortcutDesktop {
                        value: settings.game_library_install_shortcut_desktop,
                    }),
                    install_shortcut_start_menu: Some(v1::InstallShortcutStartMenu {
                        value: settings.game_library_install_shortcut_start_menu,
                    }),
                    disable_games_tab: Some(v1::DisableGamesTab {
                        value: settings.game_library_disable_games_tab,
                    }),
                }),
                // TODO: implement
                status: None,
                localization: Some(v1::LocalizationSettings {
                    locale: Some(v1::Locale {
                        locale_code: settings.localization_locale,
                    }),
                    timezone_offset: Some(v1::TimezoneOffset {
                        offset: settings.localization_timezone_offset,
                    }),
                }),
                appearance: Some(v1::AppearanceSettings {
                    theme: settings.appearance_theme,
                    developer_mode: settings.appearance_developer_mode,
                }),
            }.encode(&mut buffer).expect("Failed to encode user settings protobuf!");
        }
        ProtoType::FrecencyUserSettings(frecency) => {
            let favourite_gifs: HashMap<String, FavoriteGIF> = serde_json::from_value(frecency.favorite_gifs.unwrap_or_default())
                .unwrap_or_default();

            let sticker_frecency: HashMap<u64, FrecencyItem> = serde_json::from_value(frecency.sticker_frecency.unwrap_or_default())
                .unwrap_or_default();

            let emoii_frecency: HashMap<String, FrecencyItem> = serde_json::from_value(frecency.emoji_frecency.unwrap_or_default())
                .unwrap_or_default();

            let application_command_frecency: HashMap<String, FrecencyItem> = serde_json::from_value(frecency.application_command_frecency.unwrap_or_default())
                .unwrap_or_default();

            v1::FrecencyUserSettings {
                versions: Some(v1::Version {
                    client_version: 0,
                    server_version: 0,
                    data_version: frecency.data_version as u32,
                }),
                favorite_gifs: Some(v1::FavoriteGiFs {
                    gifs: favourite_gifs.iter().map(|(k, v)| (k.clone(), FavoriteGif {
                        format: v.format as i32,
                        src: v.clone().src,
                        width: v.width,
                        height: v.height,
                        order: v.order,
                    })).collect(),
                    hide_tooltip: frecency.favorite_gifs_hide_tooltip,
                }),
                favorite_stickers: Some(v1::FavoriteStickers {
                    sticker_ids: frecency.favorite_stickers.unwrap_or_default().iter().map(|x| *x as u64).collect(),
                }),
                sticker_frecency: Some(v1::StickerFrecency {
                    stickers: sticker_frecency.iter().map(|(k, v)| (*k, v1::FrecencyItem {
                        total_uses: v.total_uses,
                        recent_uses: v.recent_uses.clone(),
                        frecency: v.frecency,
                        score: v.score,
                    })).collect(),
                }),
                favorite_emojis: Some(v1::FavoriteEmojis {
                    emojis: frecency.favorite_emojis.unwrap_or_default(),
                }),
                emoji_frecency: Some(v1::EmojiFrecency {
                    emojis: emoii_frecency.iter().map(|(k , v)| (k.clone(), v1::FrecencyItem {
                        total_uses: v.total_uses,
                        recent_uses: v.recent_uses.clone(),
                        frecency: v.frecency,
                        score: v.score,
                    })).collect(),
                }),
                application_command_frecency: Some(v1::ApplicationCommandFrecency {
                    application_commands: application_command_frecency.iter().map(|(k , v)| (k.clone(), v1::FrecencyItem {
                        total_uses: v.total_uses,
                        recent_uses: v.recent_uses.clone(),
                        frecency: v.frecency,
                        score: v.score,
                    })).collect(),
                }),
            }.encode(&mut buffer).expect("Failed to encode FrecencyUserSettings!");
        }
    }

    BASE64_STANDARD.encode(buffer)
}