use sea_orm_migration::prelude::*;
use crate::ColumnType::{BigInteger, SmallInteger, String};
use crate::m20220101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSetting::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserSetting::User).big_integer().not_null())
                    .col(ColumnDef::new(UserSetting::DataVersion).big_integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::InboxCurrentTab).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::InboxViewedTutorial).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::UserContentDismissed).array(SmallInteger))
                    .col(ColumnDef::new(UserSetting::VoiceVideoBackgroundBlur).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::VoiceVideoAlwaysPreview).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::VoiceVideoPresetOption).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::VoiceAfkTimeout).integer().not_null().default(15))
                    .col(ColumnDef::new(UserSetting::VoiceStreamNotificationsEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::VoiceNativePhoneIntegrationEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextUseRichChatInput).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextUseThreadSidebar).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextRenderSpoilers).string().not_null().default("ON_CLICK"))
                    .col(ColumnDef::new(UserSetting::TextEmojiPickerCollapsedSections).array(String(StringLen::None)).not_null())
                    .col(ColumnDef::new(UserSetting::TextStickerPickerCollapsedSections).array(String(StringLen::None)).not_null())
                    .col(ColumnDef::new(UserSetting::TextViewImageDescriptions).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::TextShowCommandSuggestions).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextInlineAttachmentMedia).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextInlineEmbedMedia).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextGifAutoPlay).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextRenderEmbeds).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextRenderReactions).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextAnimateEmoji).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextAnimateStickers).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::TextEnableTtsCommand).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextMessageDisplayCompact).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::TextExplicitContentFilter).integer().not_null().default(2))
                    .col(ColumnDef::new(UserSetting::TextViewNsfwGuilds).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::TextConvertEmoticons).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextExpressionSuggestionsEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::TextViewNsfwCommands).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::NotificationShowInAppNotifications).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::NotificationNotifyFriendsOnGoLive).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::PrivacyAllowActivityPartyPrivacyFriends).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::PrivacyAllowActivityPartyPrivacyVoiceChannel).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSetting::PrivacyRestrictedGuildIds).array(BigInteger))
                    .col(ColumnDef::new(UserSetting::PrivacyDefaultGuildsRestricted).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::PrivacyAllowAccessibilityDetection).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::PrivacyDetectPlatformAccounts).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::PrivacyContactSyncEnabled).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::PrivacyFriendSourceFlags).integer().not_null().default(14))
                    .col(ColumnDef::new(UserSetting::PrivacyFriendDiscoveryFlags).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::PrivacyActivityRestrictedGuildIds).array(BigInteger))
                    .col(ColumnDef::new(UserSetting::PrivacyGuildActivityStatusRestrictionDefault).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::PrivacyActivityJoiningRestrictedGuildIds).array(BigInteger))
                    .col(ColumnDef::new(UserSetting::DebugRtcPanelShowVoiceStates).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::GameLibraryInstallShortcutDesktop).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::GameLibraryInstallShortcutStartMenu).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::GameLibraryDisableGamesTab).boolean().not_null().default(false))
                    .col(ColumnDef::new(UserSetting::LocalizationLocale).string().not_null().default("en-US"))
                    .col(ColumnDef::new(UserSetting::LocalizationTimezoneOffset).integer().not_null().default(420))
                    .col(ColumnDef::new(UserSetting::AppearanceTheme).integer().not_null().default(0))
                    .col(ColumnDef::new(UserSetting::AppearanceDeveloperMode).boolean().not_null().default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user-setting-user_user-id")
                            .from(UserSetting::Table, UserSetting::User)
                            .to(User::Table, User::Id)
                    )
                    .primary_key(
                        Index::create()
                            .col(UserSetting::User)
                            .col(UserSetting::DataVersion)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSetting::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserSetting {
    Table,
    User,
    DataVersion,
    InboxCurrentTab,
    InboxViewedTutorial,
    UserContentDismissed,
    VoiceVideoBackgroundBlur,
    VoiceVideoPresetOption,
    VoiceVideoAlwaysPreview,
    VoiceAfkTimeout,
    VoiceStreamNotificationsEnabled,
    VoiceNativePhoneIntegrationEnabled,
    TextUseRichChatInput,
    TextUseThreadSidebar,
    TextRenderSpoilers,
    TextEmojiPickerCollapsedSections,
    TextStickerPickerCollapsedSections,
    TextViewImageDescriptions,
    TextShowCommandSuggestions,
    TextInlineAttachmentMedia,
    TextInlineEmbedMedia,
    TextGifAutoPlay,
    TextRenderEmbeds,
    TextRenderReactions,
    TextAnimateEmoji,
    TextAnimateStickers,
    TextEnableTtsCommand,
    TextMessageDisplayCompact,
    TextExplicitContentFilter,
    TextViewNsfwGuilds,
    TextConvertEmoticons,
    TextExpressionSuggestionsEnabled,
    TextViewNsfwCommands,
    NotificationShowInAppNotifications,
    NotificationNotifyFriendsOnGoLive,
    PrivacyAllowActivityPartyPrivacyFriends,
    PrivacyAllowActivityPartyPrivacyVoiceChannel,
    PrivacyRestrictedGuildIds,
    PrivacyDefaultGuildsRestricted,
    PrivacyAllowAccessibilityDetection,
    PrivacyDetectPlatformAccounts,
    PrivacyContactSyncEnabled,
    PrivacyFriendSourceFlags,
    PrivacyFriendDiscoveryFlags,
    PrivacyActivityRestrictedGuildIds,
    PrivacyGuildActivityStatusRestrictionDefault,
    PrivacyActivityJoiningRestrictedGuildIds,
    DebugRtcPanelShowVoiceStates,
    GameLibraryInstallShortcutDesktop,
    GameLibraryInstallShortcutStartMenu,
    GameLibraryDisableGamesTab,
    LocalizationLocale,
    LocalizationTimezoneOffset,
    AppearanceTheme,
    AppearanceDeveloperMode
}