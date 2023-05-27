use std::io::Write;
use crate::gateway::schema::opcodes::{DispatchData, GatewayData, OpCodes};
use crate::gateway::schema::ready::{Consents, ConsentsEntry, ReadState, Ready, Tutorial, UserGuildSettings};
use crate::gateway::schema::{GatewayMessage};
use crate::state::SOCKET;
use axum::extract::ws::Message;
use epl_common::options::{EplOptions, Options};
use crate::AppState;
use crate::gateway::dispatch;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn dispatch_ready(user: epl_common::database::entities::user::Model, token: &String, state: &AppState) {
    let mut socket = SOCKET.get().lock().await;

    // TODO: Stub guild settings until we learn more about them
    let user_guild_settings = UserGuildSettings {
        version: 0,
        partial: false,
        entries: vec![],
    };

    let user_struct = crate::gateway::schema::ready::User {
        verified: user.acct_verified,
        username: user.username,
        purchased_flags: user.purchased_flags.unwrap_or(0),
        premium_type: user.premium_type.unwrap_or(0),
        premium: (user.premium_type.unwrap_or(0) != 0),
        phone: user.phone,
        nsfw_allowed: user.nsfw_allowed,
        // FIXME: We need to store more information about the current session
        mobile: false,
        mfa_enabled: user.mfa_enabled,
        id: user.id.to_string(),
        // TODO: pomelo related?
        global_name: None,
        flags: user.flags,
        email: user.email,
        // TODO: pomelo related?
        display_name: None,
        discriminator: user.discriminator,
        // FIXME: Same as "mobile"
        desktop: false,
        bio: user.bio.unwrap_or(String::new()),
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

    // let current_session = match get_session(&state.conn, token).await {
    //     Ok(session) => session,
    //     Err(_) => {
    //         socket
    //             .as_mut()
    //             .unwrap()
    //             .inner
    //             .send(Message::Close(Some(CloseFrame {
    //                 code: ErrorCode::UnknownError.into(),
    //                 reason: ErrorCode::UnknownError.into(),
    //             })))
    //             .await
    //             .expect("Failed to close websocket!");
    //
    //         return;
    //     }
    // };

    // TODO: Build this up eventually
    let sessions = vec![];

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

    let ready_string = serde_json::to_string(&GatewayMessage {
        s: None,
        t: Some(String::from("READY")),
        op: OpCodes::DISPATCH,
        d: Some(GatewayData::DISPATCH {
            data: Box::from(DispatchData::READY(Ready {
                version: 9,
                users: vec![],
                user_settings_proto: String::new(),
                user_guild_settings,
                user: user_struct,
                tutorial,
                sessions,
                // TODO: Need more research about this
                session_type: String::from("normal"),
                // FIXME: We should have a separate id for the session...
                session_id: String::from(""),
                resume_gateway_url: EplOptions::get().gateway_url,
                relationships: vec![],
                read_state,
                private_channels: vec![],
                merged_members: vec![],
                guilds: vec![],
                guild_join_requests: vec![],
                guild_experiments: vec![],
                geo_ordered_rtc_regions: vec![],
                friend_suggestion_count: 0,
                experiments: vec![],
                country_code: String::from("US"),
                consents,
                connected_accounts: vec![],
                auth_session_id_hash: String::from(""),
                api_code_version: 1,
                // We don't do analytics
                analytics_token: String::from(""),
            })),
        }),
    }).expect("Failed to serialize READY");

    // READY *MUST* be Zlib compressed, so we do that here
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(ready_string.as_bytes()).expect("Failed to compress READY!");

    socket
        .as_mut()
        .unwrap()
        .inner
        .send(Message::Binary(
            e.finish().expect("Failed to compress READY!")
        ))
        .await
        .expect("Failed to send REDY to client");

    drop(socket);

    dispatch::ready_supplemental::dispatch_ready_supplemental().await;
}
