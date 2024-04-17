use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use base64::prelude::*;
use prost::Message;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use serde_derive::{Deserialize, Serialize};
use tracing::debug;
use epl_common::database::entities::prelude::UserSetting;
use epl_common::database::entities::user_setting;
use epl_common::protobufs::{generate_user_proto, ProtoType};
use epl_common::protobufs::discord_protos::discord_users::v1::PreloadedUserSettings;
use crate::AppState;
use crate::authorization_extractor::SessionContext;

#[derive(Deserialize)]
pub struct SettingsProtoReq {
    settings: String,
}

#[derive(Serialize)]
pub struct SettingsProtoRes {
    settings: String,
}

pub async fn edit_settings_proto(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(proto): Path<i32>,
    Json(data): Json<SettingsProtoReq>,
) -> impl IntoResponse {
    match proto {
        1 => {
            let settings_request = BASE64_STANDARD.decode(data.settings.as_bytes());

            match settings_request {
                Ok(settings_request) => {
                    let settings_message = PreloadedUserSettings::decode(settings_request.as_slice());

                    match settings_message {
                        Ok(settings_message) => {
                            debug!("{:?}", settings_message);

                            let user_settings = UserSetting::find()
                                .filter(user_setting::Column::User.eq(session_context.user.id))
                                .one(&state.conn)
                                .await
                                .expect("Failed to access database!")
                                .expect("User doesn't have any settings!");

                            let mut user_settings_active = user_settings.clone().into_active_model();

                            user_settings_active.data_version = Set(user_settings.data_version + 1);

                            if let Some(inbox) = settings_message.inbox {
                                user_settings_active.inbox_current_tab = Set(inbox.current_tab);
                                user_settings_active.inbox_viewed_tutorial = Set(inbox.viewed_tutorial);
                            }

                            if let Some(user_content) = settings_message.user_content {
                                user_settings_active.user_content_dismissed = Set(Some(user_content.dismissed_contents.iter().map(|x| *x as i16).collect()));
                            }

                            if let Some(voice_and_video) = settings_message.voice_and_video {
                                if let Some(blur) = voice_and_video.blur {
                                    user_settings_active.voice_video_background_blur = Set(blur.use_blur);
                                }

                                user_settings_active.voice_video_preset_option = Set(voice_and_video.preset_option as i32);

                                if let Some(always_preview_video) = voice_and_video.always_preview_video {
                                    user_settings_active.voice_video_always_preview = Set(always_preview_video.value);
                                }

                                if let Some(afk_timeout) = voice_and_video.afk_timeout {
                                    user_settings_active.voice_afk_timeout = Set(afk_timeout.value as i32);
                                }

                                if let Some(stream_notifications_enabled) = voice_and_video.stream_notifications_enabled {
                                    user_settings_active.voice_stream_notifications_enabled = Set(stream_notifications_enabled.value);
                                }

                                if let Some(native_phone_integration_enabled) = voice_and_video.native_phone_integration_enabled {
                                    user_settings_active.voice_native_phone_integration_enabled = Set(native_phone_integration_enabled.value);
                                }
                            }

                            if let Some(text_and_images) = settings_message.text_and_images {
                                if let Some(use_rich_chat_input) = text_and_images.use_rich_chat_input {
                                    user_settings_active.text_use_rich_chat_input = Set(use_rich_chat_input.value);
                                }

                                if let Some(use_thread_sidebar) = text_and_images.use_thread_sidebar {
                                    user_settings_active.text_use_thread_sidebar = Set(use_thread_sidebar.value);
                                }

                                if let Some(render_spoilers) = text_and_images.render_spoilers {
                                    user_settings_active.text_render_spoilers = Set(render_spoilers.value);
                                }

                                user_settings_active.text_emoji_picker_collapsed_sections = Set(text_and_images.emoji_picker_collapsed_sections);
                                user_settings_active.text_sticker_picker_collapsed_sections = Set(text_and_images.sticker_picker_collapsed_sections);

                                if let Some(view_image_descriptions) = text_and_images.view_image_descriptions {
                                    user_settings_active.text_view_image_descriptions = Set(view_image_descriptions.value);
                                }

                                if let Some(show_command_suggestions) = text_and_images.show_command_suggestions {
                                    user_settings_active.text_show_command_suggestions = Set(show_command_suggestions.value);
                                }

                                if let Some(inline_attachment_media) = text_and_images.inline_attachment_media {
                                    user_settings_active.text_inline_attachment_media = Set(inline_attachment_media.value);
                                }

                                if let Some(inline_embed_media) = text_and_images.inline_embed_media {
                                    user_settings_active.text_inline_embed_media = Set(inline_embed_media.value);
                                }

                                if let Some(gif_auto_play) = text_and_images.gif_auto_play {
                                    user_settings_active.text_gif_auto_play = Set(gif_auto_play.value);
                                }

                                if let Some(render_embeds) = text_and_images.render_embeds {
                                    user_settings_active.text_render_embeds = Set(render_embeds.value);
                                }

                                if let Some(render_reactions) = text_and_images.render_reactions {
                                    user_settings_active.text_render_reactions = Set(render_reactions.value);
                                }

                                if let Some(animate_emoji) = text_and_images.animate_emoji {
                                    user_settings_active.text_animate_emoji = Set(animate_emoji.value);
                                }

                                if let Some(animate_stickers) = text_and_images.animate_stickers {
                                    user_settings_active.text_animate_stickers = Set(animate_stickers.value as i32);
                                }

                                if let Some(enable_tts_command) = text_and_images.enable_tts_command {
                                    user_settings_active.text_enable_tts_command = Set(enable_tts_command.value);
                                }

                                if let Some(message_display_compact) = text_and_images.message_display_compact {
                                    user_settings_active.text_message_display_compact = Set(message_display_compact.value);
                                }

                                if let Some(explicit_content_filter) = text_and_images.explicit_content_filter {
                                    user_settings_active.text_explicit_content_filter = Set(explicit_content_filter.value as i32);
                                }

                                if let Some(view_nsfw_guilds) = text_and_images.view_nsfw_guilds {
                                    user_settings_active.text_view_nsfw_guilds = Set(view_nsfw_guilds.value);
                                }

                                if let Some(convert_emoticons) = text_and_images.convert_emoticons {
                                    user_settings_active.text_convert_emoticons = Set(convert_emoticons.value);
                                }

                                if let Some(expression_suggestions_enabled) = text_and_images.expression_suggestions_enabled {
                                    user_settings_active.text_expression_suggestions_enabled = Set(expression_suggestions_enabled.value);
                                }

                                if let Some(view_nsfw_commands) = text_and_images.view_nsfw_commands {
                                    user_settings_active.text_view_nsfw_commands = Set(view_nsfw_commands.value);
                                }
                            }

                            if let Some(notifications) = settings_message.notifications {
                                if let Some(show_in_app_notifications) = notifications.show_in_app_notifications {
                                    user_settings_active.notification_show_in_app_notifications = Set(show_in_app_notifications.value);
                                }

                                if let Some(notify_friends_on_go_live) = notifications.notify_friends_on_go_live {
                                    user_settings_active.notification_notify_friends_on_go_live = Set(notify_friends_on_go_live.value);
                                }
                            }

                            if let Some(privacy) = settings_message.privacy {
                                if let Some(allow_activity_party_privacy_friends) = privacy.allow_activity_party_privacy_friends {
                                    user_settings_active.privacy_allow_activity_party_privacy_friends = Set(allow_activity_party_privacy_friends.value);
                                }

                                if let Some(allow_activity_party_privacy_voice_channel) = privacy.allow_activity_party_privacy_voice_channel {
                                    user_settings_active.privacy_allow_activity_party_privacy_voice_channel = Set(allow_activity_party_privacy_voice_channel.value);
                                }

                                user_settings_active.privacy_restricted_guild_ids = Set(Some(privacy.restricted_guild_ids.iter().map(|x| *x as i64).collect()));
                                user_settings_active.privacy_default_guilds_restricted = Set(privacy.default_guilds_restricted);
                                user_settings_active.privacy_allow_accessibility_detection = Set(privacy.allow_accessibility_detection);

                                if let Some(detect_platform_accounts) = privacy.detect_platform_accounts {
                                    user_settings_active.privacy_detect_platform_accounts = Set(detect_platform_accounts.value);
                                }

                                if let Some(contact_sync_enabled) = privacy.contact_sync_enabled {
                                    user_settings_active.privacy_contact_sync_enabled = Set(contact_sync_enabled.value);
                                }

                                if let Some(friend_source_flags) = privacy.friend_source_flags {
                                    user_settings_active.privacy_friend_source_flags = Set(friend_source_flags.value as i32);
                                }

                                if let Some(friend_discovery_flags) = privacy.friend_discovery_flags {
                                    user_settings_active.privacy_friend_discovery_flags = Set(friend_discovery_flags.value as i32);
                                }

                                user_settings_active.privacy_activity_restricted_guild_ids = Set(Some(privacy.activity_restricted_guild_ids.iter().map(|x| *x as i64).collect()));
                                user_settings_active.privacy_guild_activity_status_restriction_default = Set(privacy.default_guilds_activity_restricted);
                                user_settings_active.privacy_activity_joining_restricted_guild_ids = Set(Some(privacy.activity_joining_restricted_guild_ids.iter().map(|x| *x as i64).collect()));
                            }

                            if let Some(debug) = settings_message.debug {
                                if let Some(rtc_panel_show_voice_states) = debug.rtc_panel_show_voice_states {
                                    user_settings_active.debug_rtc_panel_show_voice_states = Set(rtc_panel_show_voice_states.value);
                                }
                            }

                            if let Some(game_library) = settings_message.game_library {
                                if let Some(install_shortcut_desktop) = game_library.install_shortcut_desktop {
                                    user_settings_active.game_library_install_shortcut_desktop = Set(install_shortcut_desktop.value);
                                }

                                if let Some(install_shortcut_start_menu) = game_library.install_shortcut_start_menu {
                                    user_settings_active.game_library_install_shortcut_start_menu = Set(install_shortcut_start_menu.value);
                                }

                                if let Some(disable_games_tab) = game_library.disable_games_tab {
                                    user_settings_active.game_library_disable_games_tab = Set(disable_games_tab.value)
                                }
                            }

                            if let Some(localization) = settings_message.localization {
                                if let Some(locale) = localization.locale {
                                    user_settings_active.localization_locale = Set(locale.locale_code);
                                }

                                if let Some(timezone_offset) = localization.timezone_offset {
                                    user_settings_active.localization_timezone_offset = Set(timezone_offset.offset);
                                }
                            }

                            if let Some(appearance) = settings_message.appearance {
                                user_settings_active.appearance_theme = Set(appearance.theme);
                                user_settings_active.appearance_developer_mode = Set(appearance.developer_mode);
                            }

                            let user_settings_active = user_settings_active.insert(&state.conn).await.expect("Failed to update settings");
                            user_settings.delete(&state.conn).await.expect("Failed to delete old settings!");

                            Json(SettingsProtoRes {
                                settings: generate_user_proto(ProtoType::PreloadedUserSettings, user_settings_active),
                            }).into_response()
                        }
                        Err(_) => {
                            StatusCode::BAD_REQUEST.into_response()
                        }
                    }
                }
                Err(_) => {
                    StatusCode::BAD_REQUEST.into_response()
                }
            }
        }
        _ => {
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

pub async fn get_settings_proto(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Path(proto): Path<i32>,
) -> impl IntoResponse {
    match proto {
        1 => {
            let user_settings = UserSetting::find()
                .filter(user_setting::Column::User.eq(session_context.user.id))
                .one(&state.conn)
                .await
                .expect("Failed to access database!")
                .expect("User doesn't have any settings!");

            Json(SettingsProtoRes {
                settings: generate_user_proto(ProtoType::PreloadedUserSettings, user_settings),
            }).into_response()
        }
        _ => {
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

