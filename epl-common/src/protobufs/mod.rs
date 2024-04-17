use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use prost::Message;
use crate::database::entities::user_setting;
use crate::protobufs::discord_protos::discord_users::v1;

pub mod discord_protos {
    pub mod discord_users {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/discord_protos.discord_users.v1.rs"));
        }
    }
}

pub enum ProtoType {
    PreloadedUserSettings
}

pub fn generate_user_proto(proto: ProtoType, settings: user_setting::Model) -> String {
    let mut buffer = vec![];

    match proto {
        ProtoType::PreloadedUserSettings => {
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
                    emoji_picker_collapsed_sections: vec![],
                    sticker_picker_collapsed_sections: vec![],
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
                    restricted_guild_ids: vec![],
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
                    activity_restricted_guild_ids: vec![],
                    default_guilds_activity_restricted: settings.privacy_guild_activity_status_restriction_default,
                    activity_joining_restricted_guild_ids: vec![],
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
    }

    BASE64_STANDARD.encode(buffer)
}